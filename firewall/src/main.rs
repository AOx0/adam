#![feature(let_chains)]

use std::io::Error;
use std::ops::ControlFlow;
use std::sync::Arc;

use anyhow::{Context, Result};
use aya::maps::{Array, MapData, RingBuf};
use aya::programs::xdp::XdpLinkId;
use aya::programs::{Xdp, XdpFlags};
use aya::{include_bytes_aligned, Bpf};
use aya_log::BpfLogger;
use clap::Parser;
use firewall_common::{processor, FirewallEvent, FirewallRule, MAX_RULES};
use log::{debug, info, warn};
use message::async_bincode::tokio::AsyncBincodeStream;
use message::{FirewallRequest, FirewallResponse, Message};
use tokio::io::unix::{AsyncFd, AsyncFdReadyMutGuard};
use tokio::net::unix::SocketAddr;
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::watch::{Receiver, Sender};
use tokio::sync::{broadcast, Mutex};
use tokio::{select, signal, sync};

#[derive(Debug, Parser)]
struct Opt {
    #[clap(short, long, default_value = "eth0")]
    iface: String,
}

#[derive(Debug, Clone, Copy)]
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
    let (etx, _) = tokio::sync::broadcast::channel(100);

    let mut programs =
        aya::maps::ProgramArray::try_from(bpf.take_map("PROCESSOR").unwrap()).unwrap();

    macro_rules! register {
        ($name:expr) => {{
            let program0: &mut Xdp = bpf.program_mut($name).unwrap().try_into().unwrap();
            program0.load()?;
            // program0
        }};
        ($at:expr, $name:expr) => {{
            register!($name);
            let prog: &Xdp = bpf.program($name).unwrap().try_into().unwrap();

            programs
                .set(processor::IPV4_TCP, prog.fd().unwrap(), 0)
                .unwrap();
        }};
    }

    register!("firewall");
    register!(programs::IPV4_TCP, "ipv4_tcp");

    let bpf = Arc::new(Mutex::new(bpf));
    let opt = Arc::new(opt);

    tokio::fs::create_dir_all("/run/adam").await.unwrap();

    let event_emitting = tokio::task::spawn({
        let mut rx = rx.clone();
        let etx = etx.clone();
        async move {
            let listener = UnixListener::bind("/run/adam/firewall_events").unwrap();

            loop {
                select! {
                    Ok((s, _addr)) = listener.accept() => {
                        tokio::task::spawn({

                            let erx: broadcast::Receiver<FirewallEvent> = etx.subscribe();
                            let rx = rx.clone();
                            emit_to_suscriber(rx,erx, s)
                        });
                    }
                    _ = rx.changed() => {
                        if let State::Terminated = *rx.borrow_and_update(){
                            break;
                        }
                    },
                }
            }
        }
    });

    // Event handling
    let handle = tokio::task::spawn({
        let mut rx = rx.clone();
        let bpf = Arc::clone(&bpf);
        let mut etx = etx.clone();
        async move {
            log::info!("Starting event handler");

            log::info!("Waiting for bpf map (program start)");
            loop {
                rx.changed().await.unwrap();
                match *rx.borrow_and_update() {
                    State::Loaded => continue,
                    State::Terminated => return,
                    State::Started => break,
                }
            }

            log::info!("Got bpf map");
            let events =
                RingBuf::try_from(bpf.lock().await.take_map("FIREWALL_EVENTS").unwrap()).unwrap();
            let mut poll = AsyncFd::new(events).unwrap();

            log::info!("Starting event listener");
            loop {
                select! {
                    guard_res = poll.readable_mut() => handle_event(guard_res, &mut etx).await,
                    _ = rx.changed() => {
                        let val =  *rx.borrow_and_update();
                        if let State::Terminated = val {
                            break;
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
            log::info!("Starting IPC");

            let listener = UnixListener::bind("/run/adam/firewall").unwrap();
            let link = Arc::new(Mutex::new(None));

            loop {
                select! {
                    Ok(socket) = listener.accept() => {
                        tokio::task::spawn(handle_stream(
                            socket,
                            rx.clone(),
                            tx.clone(),
                            Arc::clone(&opt),
                            Arc::clone(&bpf),
                            Arc::clone(&link)
                        ));
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
    event_emitting.await.unwrap();

    tokio::fs::remove_dir_all("/run/adam").await.unwrap();

    Ok(())
}

async fn emit_to_suscriber(
    mut rx: Receiver<State>,
    mut erx: broadcast::Receiver<FirewallEvent>,
    s: UnixStream,
) {
    use message::async_bincode::tokio::AsyncBincodeWriter;

    let mut s: AsyncBincodeWriter<
        UnixStream,
        FirewallEvent,
        message::async_bincode::AsyncDestination,
    > = AsyncBincodeWriter::from(s).for_async();
    loop {
        select! {
            Ok(event) = erx.recv() => {
                log::info!("Relaying event");
                if futures::SinkExt::send(&mut s, event).await.is_err() {
                    break;
                };
            }
            _ = rx.changed() => {
                if let State::Terminated = *rx.borrow_and_update(){
                    break;
                }
            },
            else => {
                break;
            }
        }
    }
}

async fn handle_message(
    msg: Message,
    tx: &mut Sender<State>,
    opt: Arc<Opt>,
    bpf: Arc<Mutex<Bpf>>,
    link: Arc<Mutex<Option<XdpLinkId>>>,
    rx: Receiver<State>,
) -> Result<ControlFlow<(), Option<FirewallResponse>>> {
    Ok(match msg {
        Message::Firewall(msg) => {
            let mut guard = bpf.lock().await;
            let mut config: Array<&mut MapData, FirewallRule> =
                Array::try_from(guard.map_mut("FIREWALL_RULES").unwrap()).unwrap();

            ControlFlow::Continue(match msg {
                FirewallRequest::DisableRule(MAX_RULES..)
                | FirewallRequest::EnableRule(MAX_RULES..)
                | FirewallRequest::DeleteRule(MAX_RULES..)
                | FirewallRequest::GetRule(MAX_RULES..) => None, // Ignore out of bounds rules
                FirewallRequest::AddRule(mut rule) => {
                    rule.init = true;
                    rule.enabled = false;

                    'res: {
                        for idx in 0..MAX_RULES {
                            if let Ok(FirewallRule { init: false, .. }) = config.get(&idx, 0) {
                                rule.id = idx;
                                config.set(idx, rule, 0).unwrap();
                                break 'res Some(FirewallResponse::Id(idx));
                            }
                        }

                        Some(FirewallResponse::ListFull)
                    }
                }
                FirewallRequest::DeleteRule(idx @ 0..MAX_RULES) => {
                    if let Ok(mut rule @ FirewallRule { init: true, .. }) = config.get(&idx, 0) {
                        rule.init = false;
                        config.set(idx, rule, 0).unwrap();
                    }
                    None
                }
                action @ FirewallRequest::EnableRule(idx @ 0..MAX_RULES)
                | action @ FirewallRequest::DisableRule(idx @ 0..MAX_RULES) => {
                    let action = match action {
                        FirewallRequest::EnableRule(_) => true,
                        FirewallRequest::DisableRule(_) => false,
                        _ => unreachable!("match only branches if enable/disable"),
                    };

                    if let Ok(mut rule @ FirewallRule { init: true, .. }) = config.get(&idx, 0)
                        && rule.enabled != action
                    {
                        rule.enabled = action;
                        config.set(idx, rule, 0).unwrap();
                    }
                    None
                }
                FirewallRequest::GetRule(idx @ 0..MAX_RULES) => {
                    if let Ok(rule @ FirewallRule { init: true, .. }) = config.get(&idx, 0) {
                        return Ok(ControlFlow::Continue(Some(FirewallResponse::Rule(rule))));
                    }

                    Some(FirewallResponse::DoesNotExist)
                }
                FirewallRequest::GetRules => {
                    let rules = config
                        .iter()
                        .flatten()
                        .filter(|r| r.init)
                        .collect::<Vec<_>>();
                    Some(FirewallResponse::Rules(rules))
                }
                FirewallRequest::Status => Some(match *rx.borrow() {
                    State::Loaded | State::Terminated => {
                        FirewallResponse::Status(message::FirewallStatus::Stopped)
                    }
                    State::Started => FirewallResponse::Status(message::FirewallStatus::Running),
                }),
            })
        }
        Message::Halt => {
            let mut link = link.lock().await;
            if link.is_none() {
                log::error!("Program not running");
                return Ok(ControlFlow::Continue(None));
            }

            log::warn!("Got halt");
            let mut guard = bpf.lock().await;
            let program: &mut Xdp = guard.program_mut("firewall").unwrap().try_into()?;

            let val = link.take();
            program.detach(val.unwrap()).unwrap();
            log::warn!("State::Loaded");
            tx.send(State::Loaded).unwrap();
            ControlFlow::Continue(None)
        }
        Message::Terminate => {
            let mut link = link.lock().await;

            log::warn!("Got terminate");
            if let Some(val) = link.take() {
                let mut guard = bpf.lock().await;
                let program: &mut Xdp = guard.program_mut("firewall").unwrap().try_into()?;

                program.detach(val).unwrap();
            }

            log::warn!("State::Terminated");
            tx.send(State::Terminated).unwrap();
            ControlFlow::Break(())
        }
        Message::Start => {
            let mut link = link.lock().await;
            if link.is_some() {
                log::error!("Program already running");
                return Ok(ControlFlow::Continue(None));
            }

            log::info!("Loading bpf program");
            let mut guard = bpf.lock().await;
            let program: &mut Xdp = guard.program_mut("firewall").unwrap().try_into()?;
            *link = Some(program.attach(&opt.iface,XdpFlags::default()).context("failed to attach the XDP program with default flags - try changing XdpFlags::default() to XdpFlags::SKB_MODE")?);

            log::warn!("State::Started");
            tx.send(State::Started).unwrap();
            ControlFlow::Continue(None)
        }
    })
}

async fn handle_stream(
    (stream, _addr): (UnixStream, SocketAddr),
    mut rx: Receiver<State>,
    mut tx: Sender<State>,
    opt: Arc<Opt>,
    bpf: Arc<Mutex<Bpf>>,
    link: Arc<Mutex<Option<XdpLinkId>>>,
) -> Result<()> {
    use futures::{SinkExt, StreamExt};

    let mut s: AsyncBincodeStream<
        UnixStream,
        Message,
        FirewallResponse,
        message::async_bincode::AsyncDestination,
    > = AsyncBincodeStream::from(stream).for_async();

    loop {
        select! {
            Some(msg) = s.next() => {
                let Ok(ControlFlow::Continue(res)) =
                    handle_message(
                        msg.unwrap(),
                        &mut tx,
                        Arc::clone(&opt),
                        Arc::clone(&bpf),
                        Arc::clone(&link),
                        rx.clone()
                    ).await else {
                    break
                };

                if let Some(res) = res && s.send(res).await.is_err() {
                    break;
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

async fn handle_event(
    guard: Result<AsyncFdReadyMutGuard<'_, RingBuf<MapData>>, Error>,
    etx: &mut broadcast::Sender<FirewallEvent>,
) {
    let mut guard = guard.unwrap();
    let ring_buf = guard.get_inner_mut();
    while let Some(item) = ring_buf.next() {
        let (_, [item], _) = (unsafe { item.align_to::<FirewallEvent>() }) else {
            continue;
        };

        if let FirewallEvent::Pass = item {
            continue;
        }

        etx.send(*item).ok(); // We dont care if there are no event listeners

        info!("{:?}", item)
        // match item {
        //     FirewallEvent::Blocked(_, _) => info!("{:?}", item),
        //     FirewallEvent::Pass => {}
        // }
    }
    guard.clear_ready();
}
