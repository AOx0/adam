use std::io::Error;
use std::sync::Arc;

use anyhow::{Context, Result};
use aya::maps::{Array, MapData, RingBuf};
use aya::programs::xdp::XdpLinkId;
use aya::programs::{Xdp, XdpFlags};
use aya::{include_bytes_aligned, Bpf};
use aya_log::BpfLogger;
use clap::Parser;
use firewall_common::{Direction, FirewallAction, FirewallEvent, FirewallMatch, FirewallRule};
use log::{debug, info, warn};
use message::Message;
use tokio::io::unix::{AsyncFd, AsyncFdReadyMutGuard};
use tokio::net::unix::SocketAddr;
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::watch::{Receiver, Sender};
use tokio::sync::Mutex;
use tokio::{select, signal, sync};

#[derive(Debug, Parser)]
struct Opt {
    #[clap(short, long, default_value = "eth0")]
    iface: String,
}

enum State {
    Loaded,
    Terminated,
    Started,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let opt = Opt::parse();

    env_logger::init();

    // Bump the memlock rlimit. This is needed for older kernels that don't use the
    // new memcg based accounting, see https://lwn.net/Articles/837122/
    let rlim = libc::rlimit {
        rlim_cur: libc::RLIM_INFINITY,
        rlim_max: libc::RLIM_INFINITY,
    };
    let ret = unsafe { libc::setrlimit(libc::RLIMIT_MEMLOCK, &rlim) };
    if ret != 0 {
        debug!("remove limit on locked memory failed, ret is: {}", ret);
    }

    // This will include your eBPF object file as raw bytes at compile-time and load it at
    // runtime. This approach is recommended for most real-world use cases. If you would
    // like to specify the eBPF program at runtime rather than at compile-time, you can
    // reach for `Bpf::load_file` instead.
    #[cfg(debug_assertions)]
    let mut bpf = Bpf::load(include_bytes_aligned!(
        "../../target/bpfel-unknown-none/debug/firewall"
    ))?;
    #[cfg(not(debug_assertions))]
    let mut bpf = Bpf::load(include_bytes_aligned!(
        "../../target/bpfel-unknown-none/release/firewall"
    ))?;
    if let Err(e) = BpfLogger::init(&mut bpf) {
        // This can happen if you remove all log statements from your eBPF program.
        warn!("failed to initialize eBPF logger: {}", e);
    }
    let (tx, mut rx) = sync::watch::channel(State::Loaded);

    let bpf = Arc::new(Mutex::new(bpf));
    let opt = Arc::new(opt);

    // Event handling
    let handle = tokio::task::spawn({
        let mut rx = rx.clone();
        let bpf = Arc::clone(&bpf);
        async move {
            loop {
                rx.changed().await.unwrap();
                match *rx.borrow_and_update() {
                    State::Loaded => continue,
                    State::Terminated => return,
                    State::Started => break,
                }
            }

            let events =
                RingBuf::try_from(bpf.lock().await.take_map("FIREWALL_EVENTS").unwrap()).unwrap();
            let mut poll = AsyncFd::new(events).unwrap();
            loop {
                select! {
                    guard_res = poll.readable_mut() => handle_event(guard_res).await,
                    _ = rx.changed() => {
                        if let State::Terminated = *rx.borrow_and_update() {
                            break
                        }
                    },
                }
            }
        }
    });

    // Message handling
    let handle2 = tokio::task::spawn({
        let mut rx = rx.clone();
        let tx = tx.clone();
        async move {
            tokio::fs::create_dir_all("/run/adam").await.unwrap();
            let listener = UnixListener::bind("/run/adam/firewall").unwrap();
            let link = Arc::new(Mutex::new(None));

            loop {
                select! {
                    Ok(socket) = listener.accept() => {
                        tokio::task::spawn(handle_stream(socket, rx.clone(), tx.clone(), Arc::clone(&opt), Arc::clone(&bpf), Arc::clone(&link)));
                    },
                    _ = rx.changed() => {
                        if let State::Terminated = *rx.borrow_and_update(){
                            break;
                        }
                    },
                    else => continue,
                }
            }
        }
    });

    info!("Waiting for Ctrl-C...");

    loop {
        select! {
            _ = signal::ctrl_c() => break,
            _ = rx.changed() => {
                if let State::Terminated = *rx.borrow_and_update(){
                    break;
                }
            }
        }
    }

    tx.send(State::Terminated).unwrap();
    info!("Exiting...");

    handle.await.unwrap();
    handle2.await.unwrap();

    tokio::fs::remove_dir_all("/run/adam").await.unwrap();

    Ok(())
}

async fn handle_stream(
    (stream, addr): (UnixStream, SocketAddr),
    mut rx: Receiver<State>,
    tx: Sender<State>,
    opt: Arc<Opt>,
    bpf: Arc<Mutex<Bpf>>,
    link: Arc<Mutex<Option<XdpLinkId>>>,
) -> Result<()> {
    let _ = addr;
    let mut buf = [0u8; 4096];

    loop {
        select! {
            Ok(_) = stream.readable() => {
                match stream.try_read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        let msg: Message = message::bincode::deserialize_from(&buf[..n]).unwrap();
                        match msg {
                            Message::Terminate => {
                                let mut link = link.lock().await;
                                if link.is_none() {
                                    log::error!("Program not running");
                                    continue;
                                }

                                log::warn!("Got terminate");
                                let mut guard = bpf.lock().await;
                                let program: &mut Xdp = guard.program_mut("firewall").unwrap().try_into()?;

                                let val = link.take();
                                program.detach(val.unwrap()).unwrap();
                                tx.send(State::Terminated).unwrap();
                            },
                            Message::Start => {
                                let mut guard = bpf.lock().await;
                                let program: &mut Xdp = guard.program_mut("firewall").unwrap().try_into()?;
                                program.load()?;
                                let mut link = link.lock().await;
                                *link = Some(program.attach(&opt.iface,XdpFlags::default()).context("failed to attach the XDP program with default flags - try changing XdpFlags::default() to XdpFlags::SKB_MODE")?);

                                let mut config: Array<&mut MapData, FirewallRule> =
                                    Array::try_from(guard.map_mut("FIREWALL_RULES").unwrap()).unwrap();

                                config
                                    .set(
                                        0,
                                        FirewallRule {
                                            action: FirewallAction::Drop,
                                            // matches: FirewallMatch::Match(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))),
                                            matches: FirewallMatch::Protocol(netp::network::InetProtocol::ICMP),
                                            applies_to: Direction::Source,
                                        },
                                        0,
                                    )
                                    .unwrap();

                                tx.send(State::Started).unwrap();
                            },
                        }

                    }
                    Err(err) if err.kind() == tokio::io::ErrorKind::WouldBlock => continue,
                    Err(err) => println!("ERR :: {err:?}"),
                }
            }
            _ = rx.changed() => {
                if let State::Terminated = *rx.borrow_and_update(){
                    break;
                }
            },
            else => break,
        }
    }

    Ok(())
}

async fn handle_event(guard: Result<AsyncFdReadyMutGuard<'_, RingBuf<MapData>>, Error>) {
    let mut guard = guard.unwrap();
    let ring_buf = guard.get_inner_mut();
    while let Some(item) = ring_buf.next() {
        let (_, [item], _) = (unsafe { item.align_to::<FirewallEvent>() }) else {
            continue;
        };

        match item {
            FirewallEvent::Blocked(_) => info!("{:?}", item),
            FirewallEvent::Pass => {}
        }
    }
    guard.clear_ready();
}
