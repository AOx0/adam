use std::io::Error;
use std::net::{IpAddr, Ipv4Addr};

use anyhow::Context;
use aya::maps::{Array, MapData, RingBuf};
use aya::programs::{Xdp, XdpFlags};
use aya::{include_bytes_aligned, Bpf};
use aya_log::BpfLogger;
use clap::Parser;
use firewall_common::{Direction, FirewallAction, FirewallEvent, FirewallMatch, FirewallRule};
use log::{debug, info, warn};
use tokio::io::unix::{AsyncFd, AsyncFdReadyMutGuard};
use tokio::{select, signal};

#[derive(Debug, Parser)]
struct Opt {
    #[clap(short, long, default_value = "eth0")]
    iface: String,
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
    let program: &mut Xdp = bpf.program_mut("firewall").unwrap().try_into()?;
    program.load()?;
    program.attach(&opt.iface, XdpFlags::default())
        .context("failed to attach the XDP program with default flags - try changing XdpFlags::default() to XdpFlags::SKB_MODE")?;

    let mut config: Array<&mut MapData, FirewallRule> =
        Array::try_from(bpf.map_mut("FIREWALL_RULES").unwrap()).unwrap();

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

    let notify = std::sync::Arc::new(tokio::sync::Notify::new());

    let events = RingBuf::try_from(bpf.take_map("FIREWALL_EVENTS").unwrap()).unwrap();
    let mut poll = AsyncFd::new(events).unwrap();

    async fn handle_event<'a>(guard: Result<AsyncFdReadyMutGuard<'a, RingBuf<MapData>>, Error>) {
        let mut guard = guard.unwrap();
        let ring_buf = guard.get_inner_mut();
        while let Some(item) = ring_buf.next() {
            let (_, [item], _) = (unsafe { item.align_to::<FirewallEvent>() }) else {
                continue;
            };

            match item {
                FirewallEvent::Blocked(_) => println!("{:?}", item),
                FirewallEvent::Pass => {}
            }
        }
        guard.clear_ready();
    }

    let handle = tokio::task::spawn({
        let notify = notify.clone();
        async move {
            loop {
                select! {
                    guard_res = poll.readable_mut() => handle_event(guard_res).await,
                    _ = notify.notified() => break,
                }
            }
        }
    });

    info!("Waiting for Ctrl-C...");
    signal::ctrl_c().await?;
    info!("Exiting...");

    notify.notify_one();
    handle.await.unwrap();

    Ok(())
}
