#![feature(let_chains)]

use std::env;
use std::fmt::Debug;
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
use diesel::sqlite::Sqlite;
use diesel_async::async_connection_wrapper::AsyncConnectionWrapper;
use diesel_async::sync_connection_wrapper::SyncConnectionWrapper;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use dotenv::dotenv;
use firewall_common::{processor, FirewallEvent, FirewallRule, MAX_RULES};
use log::{debug, info, warn};
use message::async_bincode::tokio::{AsyncBincodeReader, AsyncBincodeStream};
use message::{FirewallRequest, FirewallResponse, Message};
use serde::{Deserialize, Serialize};
use tokio::io::unix::{AsyncFd, AsyncFdReadyMutGuard};
use tokio::net::unix::SocketAddr;
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::watch::{Receiver, Sender};
use tokio::sync::{broadcast, Mutex};
use tokio::{select, signal, sync};

use diesel::prelude::*;
use diesel_async::{AsyncConnection, RunQueryDsl};

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

diesel::table! {
    rules (id) {
        id -> Integer,
        // name -> Text,
        // description-> Text,
        rule -> Blob,
    }
}

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

/// From: <https://github.com/weiznich/diesel_async/blob/5b8262b86d8ed0e13adbbc4aee39500b9931ef8d/examples/sync-wrapper/src/main.rs#L36>
async fn run_migrations<A>(async_connection: A) -> Result<(), Box<dyn std::error::Error>>
where
    A: AsyncConnection<Backend = Sqlite> + 'static,
{
    info!("Running sqlite migrations");
    let mut async_wrapper: AsyncConnectionWrapper<A> =
        AsyncConnectionWrapper::from(async_connection);

    tokio::task::spawn_blocking(move || {
        async_wrapper.run_pending_migrations(MIGRATIONS).unwrap();
    })
    .await
    .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}

async fn init_db() -> SyncConnectionWrapper<SqliteConnection> {
    let db = get_db().await;

    run_migrations(db).await.unwrap();
    get_db().await
}

async fn get_db() -> SyncConnectionWrapper<SqliteConnection> {
    dotenv().ok();

    tokio::fs::create_dir_all("/var/lib/adam").await.unwrap();
    let db = env::var("DATABASE_URL").unwrap_or("file:///var/lib/adam/firewall.db".to_string());

    SyncConnectionWrapper::<SqliteConnection>::establish(&db)
        .await
        .unwrap()
}

#[derive(Serialize, Deserialize, Identifiable, Queryable, Insertable)]
#[diesel(table_name = rules)]
struct StoredRule {
    pub id: i32,
    // pub name: String,
    // pub description: String,
    pub rule: Vec<u8>,
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
            program0
        }};
        ($name:expr, $at:expr) => {{
            register!($name);
            let prog: &Xdp = bpf.program($name).unwrap().try_into().unwrap();

            programs
                .set(processor::IPV4_TCP, prog.fd().unwrap(), 0)
                .unwrap();
        }};
    }

    let mut config: Array<&mut MapData, FirewallRule> =
        Array::try_from(bpf.map_mut("FIREWALL_RULES").unwrap()).unwrap();
    let db = &mut init_db().await;

    let rules: Vec<StoredRule> = rules::table.load(db).await.unwrap();
    for rule in rules {
        let deserialize_from: FirewallRule =
            bincode::deserialize_from(rule.rule.as_slice()).unwrap();
        config.set(rule.id as u32, deserialize_from, 0).unwrap();
    }

    register!("firewall");
    register!("ipv4_tcp", programs::IPV4_TCP);

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
                                let mut db = get_db().await;

                                diesel::insert_into(rules::table)
                                    .values(StoredRule {
                                        id: idx as i32,
                                        rule: bincode::serialize(&rule).unwrap(),
                                    })
                                    .execute(&mut db)
                                    .await
                                    .unwrap();

                                break 'res Some(FirewallResponse::Id(idx));
                            }
                        }

                        Some(FirewallResponse::ListFull)
                    }
                }
                FirewallRequest::DeleteRule(idx @ 0..MAX_RULES) => {
                    if let Ok(mut rule @ FirewallRule { init: true, .. }) = config.get(&idx, 0) {
                        rule.init = false;
                        let mut db = get_db().await;

                        diesel::delete(rules::table.filter(rules::dsl::id.eq(idx as i32)))
                            .execute(&mut db)
                            .await
                            .unwrap();

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

                        let mut db = get_db().await;

                        diesel::update(rules::table.filter(rules::dsl::id.eq(idx as i32)))
                            .set(rules::dsl::rule.eq(bincode::serialize(&rule).unwrap()))
                            .execute(&mut db)
                            .await
                            .unwrap();

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
