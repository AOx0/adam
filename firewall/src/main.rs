#![feature(let_chains)]

use std::env;
use std::fmt::Debug;
use std::io::Error;
use std::ops::{ControlFlow, DerefMut};
use std::sync::Arc;

use anyhow::{Context, Result};
use aya::maps::{Array, MapData, RingBuf};
use aya::programs::xdp::XdpLinkId;
use aya::programs::{Xdp, XdpFlags};
use aya::{include_bytes_aligned, Ebpf};
use aya_log::EbpfLogger;
use chrono::NaiveDateTime;
use clap::Parser;
use diesel::sqlite::Sqlite;
use diesel_async::async_connection_wrapper::AsyncConnectionWrapper;
use diesel_async::sync_connection_wrapper::SyncConnectionWrapper;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use dotenv::dotenv;
use firewall_common::{processor, Event, Rule, StoredEventDecoded, StoredRuleDecoded, MAX_RULES};
use futures::SinkExt;
use log::{debug, info, warn};
use message::async_bincode::tokio::AsyncBincodeStream;
use message::firewall::*;
use message::Message;
use serde::{Deserialize, Serialize};
use std::io::ErrorKind;
use tokio::io::unix::{AsyncFd, AsyncFdReadyMutGuard};
use tokio::net::unix::SocketAddr;
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::watch::{Receiver, Sender};
use tokio::sync::{broadcast, Mutex, OnceCell};
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
    events (id) {
        id -> Integer,
        time -> Timestamp,
        event -> Blob,
    }
}

diesel::table! {
    rules (id) {
        id -> Integer,
        name -> Text,
        description-> Text,
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

async fn init_db() -> Arc<Mutex<SyncConnectionWrapper<SqliteConnection>>> {
    tokio::fs::create_dir_all("/var/lib/adam").await.unwrap();
    let db = env::var("DATABASE_URL").unwrap_or("file:///var/lib/adam/firewall.db".to_string());
    let db = SyncConnectionWrapper::<SqliteConnection>::establish(&db)
        .await
        .unwrap();

    run_migrations(db).await.unwrap();

    get_db().await
}

static DB: OnceCell<Arc<Mutex<SyncConnectionWrapper<SqliteConnection>>>> = OnceCell::const_new();

async fn get_db() -> Arc<Mutex<SyncConnectionWrapper<SqliteConnection>>> {
    dotenv().ok();

    DB.get_or_init(|| async {
        let db = env::var("DATABASE_URL").unwrap_or("file:///var/lib/adam/firewall.db".to_string());
        Arc::new(Mutex::new(
            SyncConnectionWrapper::<SqliteConnection>::establish(&db)
                .await
                .unwrap(),
        ))
    })
    .await
    .clone()
}

#[derive(Serialize, Deserialize, Identifiable, Queryable, Insertable, Selectable)]
#[diesel(table_name = rules)]
struct StoredRule {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub rule: Vec<u8>,
}

#[derive(Serialize, Deserialize, Identifiable, Queryable, Insertable, Selectable)]
#[diesel(table_name = rules)]
struct StoredRuleRef<'a> {
    pub id: i32,
    pub name: &'a str,
    pub description: &'a str,
    pub rule: &'a [u8],
}

#[derive(Serialize, Deserialize, Queryable, Insertable, Selectable)]
#[diesel(table_name = events)]
struct StoredEventRef<'a> {
    pub time: NaiveDateTime,
    pub event: &'a [u8],
}

#[derive(Serialize, Deserialize, Queryable, Insertable, Selectable)]
#[diesel(table_name = events)]
struct StoredEvent {
    pub time: NaiveDateTime,
    pub event: Vec<u8>,
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
    let mut bpf = Ebpf::load(include_bytes_aligned!(
        "../../target/bpfel-unknown-none/debug/firewall"
    ))?;
    #[cfg(not(debug_assertions))]
    let mut bpf = Ebpf::load(include_bytes_aligned!(
        "../../target/bpfel-unknown-none/release/firewall"
    ))?;
    if let Err(e) = EbpfLogger::init(&mut bpf) {
        // This can happen if you remove all log statements from your eBPF program.
        warn!("failed to initialize eBPF logger: {}", e);
    }

    let (tx, mut rx) = sync::watch::channel(State::Loaded);
    let (etx, _) = tokio::sync::broadcast::channel(100);

    let mut programs =
        aya::maps::ProgramArray::try_from(bpf.take_map("PROCESSOR").unwrap()).unwrap();

    macro_rules! register {
        ($name:expr) => {{
            info!("Loading {}", $name);
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

    let mut config: Array<&mut MapData, Rule> =
        Array::try_from(bpf.map_mut("FIREWALL_RULES").unwrap()).unwrap();
    let db = &mut init_db().await;

    {
        let mut db = db.lock().await;

        let rules: Vec<StoredRule> = rules::table.load(db.deref_mut()).await.unwrap();
        for rule in rules {
            let deserialize_from: Rule = bincode::deserialize_from(rule.rule.as_slice()).unwrap();
            config.set(rule.id as u32, deserialize_from, 0).unwrap();
        }
    };

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
                            let erx: broadcast::Receiver<LogKind> = etx.subscribe();
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
    mut erx: broadcast::Receiver<LogKind>,
    s: UnixStream,
) {
    use message::async_bincode::tokio::AsyncBincodeWriter;

    let mut s: AsyncBincodeWriter<UnixStream, LogKind, message::async_bincode::AsyncDestination> =
        AsyncBincodeWriter::from(s).for_async();

    loop {
        select! {
        Ok(event) = erx.recv() => {
            match SinkExt::send(&mut s, event).await {
                Ok(_) => continue,
                Err(e) => match *e {
                    bincode::ErrorKind::Io(ref e) if e.kind() == ErrorKind::UnexpectedEof || e.kind() == ErrorKind::BrokenPipe => {
                        log::info!("Channel closed normally (EOF/broken pipe)");
                        break;
                    }
                    _ => {
                        log::error!("Fatal error while sending event: {}", e);
                        panic!("Fatal error in event relay: {}", e);
                    }
                }
            }
        },
        _ = rx.changed() => {
            if let State::Terminated = *rx.borrow_and_update() {
                break;
            }
        }
        }
    }
}

async fn handle_message(
    msg: Message,
    tx: &mut Sender<State>,
    opt: Arc<Opt>,
    bpf: Arc<Mutex<Ebpf>>,
    link: Arc<Mutex<Option<XdpLinkId>>>,
    rx: Receiver<State>,
) -> Result<ControlFlow<(), Option<Response>>> {
    Ok(match msg {
        Message::Firewall(msg) => {
            let mut guard = bpf.lock().await;
            let mut config: Array<&mut MapData, Rule> =
                Array::try_from(guard.map_mut("FIREWALL_RULES").unwrap()).unwrap();

            ControlFlow::Continue(match msg {
                Request::DisableRule(MAX_RULES..)
                | Request::EnableRule(MAX_RULES..)
                | Request::ToggleRule(MAX_RULES..) => {
                    Some(Response::RuleChange(RuleChange::NoSuchRule))
                }
                Request::DeleteRule(MAX_RULES..) | Request::GetRule(MAX_RULES..) => None, // Ignore out of bounds rules
                Request::AddRule(meta) => {
                    let mut rule = meta.rule;

                    rule.init = true;
                    rule.enabled = false;

                    'res: {
                        for idx in 0..MAX_RULES {
                            if let Ok(Rule { init: false, .. }) = config.get(&idx, 0) {
                                rule.id = idx;
                                config.set(idx, rule, 0).unwrap();
                                let mut buffer = [0u8; std::mem::size_of::<Rule>()];

                                bincode::serialize_into(&mut buffer[..], &rule).unwrap();
                                diesel::insert_into(rules::table)
                                    .values(StoredRuleRef {
                                        id: idx as i32,
                                        rule: &buffer,
                                        name: &meta.name,
                                        description: &meta.description,
                                    })
                                    .execute(get_db().await.lock().await.deref_mut())
                                    .await
                                    .unwrap();

                                break 'res Some(Response::Id(idx));
                            }
                        }

                        Some(Response::ListFull)
                    }
                }
                Request::DeleteRule(idx @ 0..MAX_RULES) => {
                    if let Ok(mut rule @ Rule { init: true, .. }) = config.get(&idx, 0) {
                        rule.init = false;

                        diesel::delete(rules::table.filter(rules::dsl::id.eq(idx as i32)))
                            .execute(get_db().await.lock().await.deref_mut())
                            .await
                            .unwrap();

                        config.set(idx, rule, 0).unwrap();
                    }
                    None
                }
                action @ Request::EnableRule(idx @ 0..MAX_RULES)
                | action @ Request::DisableRule(idx @ 0..MAX_RULES)
                | action @ Request::ToggleRule(idx @ 0..MAX_RULES) => {
                    #[derive(PartialEq, Eq)]
                    enum Action {
                        Toggle,
                        Enable,
                        Disable,
                    }

                    impl Action {
                        fn as_bool(&self) -> Option<bool> {
                            match self {
                                Action::Toggle => None,
                                Action::Enable => Some(true),
                                Action::Disable => Some(false),
                            }
                        }
                    }

                    let action = match action {
                        Request::EnableRule(_) => Action::Enable,
                        Request::DisableRule(_) => Action::Disable,
                        Request::ToggleRule(_) => Action::Toggle,
                        _ => unreachable!("match only branches if enable/disable"),
                    };

                    'a: {
                        if let Ok(mut rule @ Rule { init: true, .. }) = config.get(&idx, 0) {
                            if action.as_bool().unwrap_or(!rule.enabled) == rule.enabled {
                                break 'a Some(Response::RuleChange(RuleChange::NoChangeRequired(
                                    if rule.enabled {
                                        RuleStatus::Active
                                    } else {
                                        RuleStatus::Inactive
                                    },
                                )));
                            }

                            rule.enabled = action.as_bool().unwrap_or(!rule.enabled);

                            diesel::update(rules::table.filter(rules::dsl::id.eq(idx as i32)))
                                .set(rules::dsl::rule.eq(bincode::serialize(&rule).unwrap()))
                                .execute(get_db().await.lock().await.deref_mut())
                                .await
                                .unwrap();

                            config.set(idx, rule, 0).unwrap();

                            Some(Response::RuleChange(RuleChange::Change(if rule.enabled {
                                RuleStatus::Active
                            } else {
                                RuleStatus::Inactive
                            })))
                        } else {
                            Some(Response::RuleChange(RuleChange::NoSuchRule))
                        }
                    }
                }
                Request::GetRule(idx @ 0..MAX_RULES) => {
                    if let Ok(rule @ Rule { init: true, .. }) = config.get(&idx, 0) {
                        let meta = rules::table
                            .filter(rules::id.eq(idx as i32))
                            .first::<StoredRule>(get_db().await.lock().await.deref_mut())
                            .await
                            .unwrap();

                        return Ok(ControlFlow::Continue(Some(Response::Rule(
                            StoredRuleDecoded {
                                id: rule.id as i32,
                                name: meta.name,
                                description: meta.description,
                                rule,
                            },
                        ))));
                    }

                    Some(Response::DoesNotExist)
                }
                Request::GetRules => {
                    let rules = config
                        .iter()
                        .flatten()
                        .enumerate()
                        .filter(|r| r.1.init)
                        .collect::<Vec<_>>();

                    let res = futures::future::join_all(rules.iter().map(|rule| {
                        let id = rule.0 as i32;
                        async move {
                            rules::table
                                .filter(rules::id.eq(id))
                                .first::<StoredRule>(get_db().await.lock().await.deref_mut())
                                .await
                                .unwrap()
                        }
                    }))
                    .await;

                    let rules = res
                        .into_iter()
                        .map(|s| StoredRuleDecoded {
                            id: s.id,
                            description: s.description,
                            name: s.name,
                            rule: rules.iter().find(|r| r.0 as i32 == s.id).unwrap().1,
                        })
                        .collect::<Vec<_>>();

                    Some(Response::Rules(rules))
                }
                Request::Status => Some(match *rx.borrow() {
                    State::Loaded | State::Terminated => Response::Status(Status::Stopped),
                    State::Started => Response::Status(Status::Running),
                }),
                Request::GetEvents(event_query) => {
                    let events = match event_query {
                        message::EventQuery::All => {
                            events::table
                                .select(StoredEvent::as_select())
                                .load::<StoredEvent>(get_db().await.lock().await.deref_mut())
                                .await
                        }
                        message::EventQuery::Last(duration) => {
                            let since = chrono::Local::now().naive_utc() - duration;

                            events::table
                                .filter(events::time.ge(since))
                                .select(StoredEvent::as_select())
                                .load::<StoredEvent>(get_db().await.lock().await.deref_mut())
                                .await
                        }
                        message::EventQuery::Since(datetime) => {
                            events::table
                                .filter(events::time.ge(datetime))
                                .select(StoredEvent::as_select())
                                .load::<StoredEvent>(get_db().await.lock().await.deref_mut())
                                .await
                        }
                    }
                    .unwrap();

                    let b = events
                        .into_iter()
                        .map(|e| StoredEventDecoded {
                            time: e.time,
                            event: bincode::deserialize_from(e.event.as_slice()).unwrap(),
                        })
                        .collect();

                    Some(Response::Events(b))
                }
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
    bpf: Arc<Mutex<Ebpf>>,
    link: Arc<Mutex<Option<XdpLinkId>>>,
) -> Result<()> {
    use futures::{SinkExt, StreamExt};

    let mut s: AsyncBincodeStream<
        UnixStream,
        Message,
        Response,
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
    etx: &mut broadcast::Sender<LogKind>,
) {
    let mut guard = guard.unwrap();
    let ring_buf = guard.get_inner_mut();
    let mut buffer = [0u8; std::mem::size_of::<Event>()];

    while let Some(item) = ring_buf.next() {
        let (_, [event], _) = (unsafe { item.align_to::<Event>() }) else {
            continue;
        };

        let time = chrono::Local::now().naive_utc();
        let stored = StoredEventDecoded {
            time,
            event: *event,
        };

        etx.send(LogKind::Event(stored)).ok(); // We dont care if there are no event listeners

        if let Event::Blocked { .. } = event {
            info!("{:?}", event);
        }

        bincode::serialize_into(&mut buffer[..], event).unwrap();
        diesel::insert_into(events::table)
            .values(StoredEventRef {
                time,
                event: &buffer,
            })
            .execute(get_db().await.lock().await.deref_mut())
            .await
            .unwrap();
    }
    guard.clear_ready();
}
