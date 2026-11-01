//! D-Bus contract tests for the Fluid Island core service.
//!
//! These tests start a private dbus-daemon, serve the real
//! `FluidIslandInterface` on it, and exercise the D-Bus contract from a
//! client. No part of the real desktop session is touched.

use fluid_island::ipc::{FluidIslandInterface, IpcEvent, SharedState};
use fluid_island::state::IslandState;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc, Mutex,
};
use std::thread;
use std::time::Duration;
use zbus::connection::Builder;
use zbus::fdo::{RequestNameFlags, RequestNameReply};

const BUS_NAME: &str = "org.fluidisland.Core";
const OBJECT_PATH: &str = "/org/fluidisland/Core";
const IFACE_NAME: &str = "org.fluidisland.Core";

/// Counter for unique socket paths per dbus-daemon instance.
static BUS_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Start a private dbus-daemon and return its bus address plus the child handle.
/// Each instance gets a unique socket path so parallel tests do not collide.
fn start_private_bus() -> Result<(String, std::process::Child), Box<dyn std::error::Error>> {
    let n = BUS_COUNTER.fetch_add(1, Ordering::Relaxed);
    let socket_path = format!("/tmp/fluid-island-test-{}-{}.sock", std::process::id(), n);

    // Remove any stale socket from a previous run.
    let _ = std::fs::remove_file(&socket_path);

    let mut child = Command::new("dbus-daemon")
        .args([
            "--session",
            "--address",
            &format!("unix:path={}", socket_path),
            "--print-address=1",
            "--nofork",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;

    // Read the actual address the daemon is listening on from stdout.
    let stdout = child
        .stdout
        .take()
        .ok_or("dbus-daemon produced no stdout")?;
    let mut reader = BufReader::new(stdout);
    let mut address = String::new();
    reader.read_line(&mut address)?;
    let address = address.trim().to_string();

    if address.is_empty() {
        return Err("dbus-daemon returned an empty address".into());
    }

    Ok((address, child))
}

/// Build a `FluidIslandInterface` that records every IPC event it emits.
fn test_interface(shared: SharedState, events: Arc<Mutex<Vec<IpcEvent>>>) -> FluidIslandInterface {
    FluidIslandInterface {
        shared,
        on_event: Arc::new(move |event| {
            events.lock().unwrap().push(event);
        }),
    }
}

/// Run a service on the given bus address until the connection closes.
/// Returns the join handle and a receiver for startup/readiness signals.
fn run_service(
    address: String,
    shared: SharedState,
    events: Arc<Mutex<Vec<IpcEvent>>>,
) -> (
    std::thread::JoinHandle<()>,
    mpsc::Receiver<Result<(), String>>,
) {
    let (tx, rx) = mpsc::channel();
    let handle = thread::spawn(move || {
        let result = zbus::block_on(async {
            let connection = match Builder::address(address.as_str()).build().await {
                Ok(c) => c,
                Err(err) => return Err(format!("connect to bus: {err}")),
            };

            let interface = test_interface(shared, events);

            if let Err(err) = connection.object_server().at(OBJECT_PATH, interface).await {
                return Err(format!("register object: {err}"));
            }

            if let Err(err) = connection.request_name(BUS_NAME).await {
                return Err(format!("acquire bus name: {err}"));
            }

            // Signal readiness before blocking on connection.closed().
            tx.send(Ok(())).ok();

            let _ = connection.closed().await;
            Ok(())
        });
        tx.send(result).ok();
    });
    (handle, rx)
}

/// Connect a client to the private bus, waiting for it to become available.
async fn connect_client(address: &str) -> zbus::Connection {
    let mut last_err = None;
    for _ in 0..100 {
        match Builder::address(address).build().await {
            Ok(c) => return c,
            Err(err) => {
                last_err = Some(err);
                thread::sleep(Duration::from_millis(50));
            }
        }
    }
    panic!("failed to connect to bus after 100 attempts: {last_err:?}");
}

/// Call `GetState` and return the state string.
async fn get_state(client: &zbus::Connection) -> String {
    let reply = client
        .call_method(
            Some(BUS_NAME),
            OBJECT_PATH,
            Some(IFACE_NAME),
            "GetState",
            &(),
        )
        .await
        .expect("GetState call failed");
    let (state,): (String,) = reply
        .body()
        .deserialize()
        .expect("failed to deserialize GetState reply");
    state
}

/// Call `Toggle` and return the new state string.
async fn toggle(client: &zbus::Connection) -> String {
    let reply = client
        .call_method(Some(BUS_NAME), OBJECT_PATH, Some(IFACE_NAME), "Toggle", &())
        .await
        .expect("Toggle call failed");
    let (state,): (String,) = reply
        .body()
        .deserialize()
        .expect("failed to deserialize Toggle reply");
    state
}

#[test]
fn initial_state_and_get_state() {
    let (address, daemon) = start_private_bus().expect("failed to start dbus-daemon");
    let _guard = BusGuard(Some(daemon));

    let shared = SharedState {
        state: Arc::new(Mutex::new(IslandState::Collapsed)),
        attached: Arc::new(AtomicBool::new(false)),
    };
    let events = Arc::new(Mutex::new(Vec::new()));
    let (_handle, ready) = run_service(address.clone(), shared, events.clone());

    // Wait for the service to signal readiness.
    ready
        .recv_timeout(Duration::from_secs(5))
        .expect("service failed to start")
        .expect("service failed to register on bus");

    zbus::block_on(async {
        let client = connect_client(&address).await;
        let state = get_state(&client).await;
        assert_eq!(state, "collapsed", "core must start collapsed");
    });
}

#[test]
fn toggle_changes_state() {
    let (address, daemon) = start_private_bus().expect("failed to start dbus-daemon");
    let _guard = BusGuard(Some(daemon));

    let shared = SharedState {
        state: Arc::new(Mutex::new(IslandState::Collapsed)),
        attached: Arc::new(AtomicBool::new(false)),
    };
    let events = Arc::new(Mutex::new(Vec::new()));
    let (_handle, ready) = run_service(address.clone(), shared, events.clone());

    ready
        .recv_timeout(Duration::from_secs(5))
        .expect("service failed to start")
        .expect("service failed to register on bus");

    zbus::block_on(async {
        let client = connect_client(&address).await;

        assert_eq!(get_state(&client).await, "collapsed");

        let new_state = toggle(&client).await;
        assert_eq!(new_state, "expanded", "Toggle must flip to expanded");

        assert_eq!(get_state(&client).await, "expanded");

        let new_state = toggle(&client).await;
        assert_eq!(new_state, "collapsed", "Toggle must flip back to collapsed");

        assert_eq!(get_state(&client).await, "collapsed");

        // The service must have emitted StateChanged events via on_event.
        let captured = events.lock().unwrap();
        assert!(
            captured
                .iter()
                .any(|e| matches!(e, IpcEvent::StateChanged(s) if *s == IslandState::Expanded)),
            "on_event must receive StateChanged(expanded)"
        );
        assert!(
            captured
                .iter()
                .any(|e| matches!(e, IpcEvent::StateChanged(s) if *s == IslandState::Collapsed)),
            "on_event must receive StateChanged(collapsed)"
        );
    });
}

#[test]
fn attach_detach_are_idempotent() {
    let (address, daemon) = start_private_bus().expect("failed to start dbus-daemon");
    let _guard = BusGuard(Some(daemon));

    let shared = SharedState {
        state: Arc::new(Mutex::new(IslandState::Collapsed)),
        attached: Arc::new(AtomicBool::new(false)),
    };
    let events = Arc::new(Mutex::new(Vec::new()));
    let (_handle, ready) = run_service(address.clone(), shared, events.clone());

    ready
        .recv_timeout(Duration::from_secs(5))
        .expect("service failed to start")
        .expect("service failed to register on bus");

    zbus::block_on(async {
        let client = connect_client(&address).await;

        // First Attach returns current state.
        let reply = client
            .call_method(Some(BUS_NAME), OBJECT_PATH, Some(IFACE_NAME), "Attach", &())
            .await
            .expect("Attach call failed");
        let (state,): (String,) = reply
            .body()
            .deserialize()
            .expect("failed to deserialize Attach reply");
        assert_eq!(state, "collapsed", "Attach must return current state");

        // Second Attach is idempotent.
        let reply = client
            .call_method(Some(BUS_NAME), OBJECT_PATH, Some(IFACE_NAME), "Attach", &())
            .await
            .expect("second Attach call failed");
        let (state,): (String,) = reply
            .body()
            .deserialize()
            .expect("failed to deserialize second Attach reply");
        assert_eq!(
            state, "collapsed",
            "repeat Attach must return current state"
        );

        // Detach.
        client
            .call_method(Some(BUS_NAME), OBJECT_PATH, Some(IFACE_NAME), "Detach", &())
            .await
            .expect("Detach call failed");

        // Second Detach is idempotent.
        client
            .call_method(Some(BUS_NAME), OBJECT_PATH, Some(IFACE_NAME), "Detach", &())
            .await
            .expect("second Detach call failed");

        // State is unchanged by Attach/Detach.
        assert_eq!(get_state(&client).await, "collapsed");

        let captured = events.lock().unwrap();
        assert!(
            captured
                .iter()
                .any(|e| matches!(e, IpcEvent::AttachmentChanged(true))),
            "on_event must receive AttachmentChanged(true)"
        );
        assert!(
            captured
                .iter()
                .any(|e| matches!(e, IpcEvent::AttachmentChanged(false))),
            "on_event must receive AttachmentChanged(false)"
        );
    });
}

#[test]
fn service_ownership_conflict_is_reported() {
    let (address, daemon) = start_private_bus().expect("failed to start dbus-daemon");
    let _guard = BusGuard(Some(daemon));

    let shared = SharedState {
        state: Arc::new(Mutex::new(IslandState::Collapsed)),
        attached: Arc::new(AtomicBool::new(false)),
    };
    let events = Arc::new(Mutex::new(Vec::new()));
    let (_handle, ready) = run_service(address.clone(), shared, events.clone());

    ready
        .recv_timeout(Duration::from_secs(5))
        .expect("service failed to start")
        .expect("service failed to register on bus");

    zbus::block_on(async {
        let _client = connect_client(&address).await;
        // A second service trying to acquire the same name should not become primary owner.
        let result = Builder::address(address.as_str())
            .build()
            .await
            .expect("failed to connect second service")
            .request_name_with_flags(BUS_NAME, RequestNameFlags::DO_NOT_QUEUE)
            .await;

        // The bus may return RequestNameReply::Exists or an error (NameTaken),
        // but it must not return PrimaryOwner.
        match result {
            Ok(reply) => {
                assert_ne!(
                    reply,
                    RequestNameReply::PrimaryOwner,
                    "a second service must not become primary owner of the bus name"
                );
            }
            Err(err) => {
                // NameTaken or similar error is acceptable: the name is already owned.
                let msg = err.to_string();
                assert!(
                    msg.contains("NameTaken") || msg.contains("exists") || msg.contains("taken"),
                    "unexpected error from request_name_with_flags: {msg}"
                );
            }
        }
    });
}

#[test]
fn state_survives_client_disconnect_and_reconnect() {
    let (address, daemon) = start_private_bus().expect("failed to start dbus-daemon");
    let _guard = BusGuard(Some(daemon));

    let shared = SharedState {
        state: Arc::new(Mutex::new(IslandState::Collapsed)),
        attached: Arc::new(AtomicBool::new(false)),
    };
    let events = Arc::new(Mutex::new(Vec::new()));
    let (_handle, ready) = run_service(address.clone(), shared, events.clone());

    ready
        .recv_timeout(Duration::from_secs(5))
        .expect("service failed to start")
        .expect("service failed to register on bus");

    // First client: toggle to expanded, then disconnect.
    {
        zbus::block_on(async {
            let client = connect_client(&address).await;
            assert_eq!(toggle(&client).await, "expanded");
        });
    }

    // Second client: state should still be expanded after first client disconnects.
    zbus::block_on(async {
        let client = connect_client(&address).await;
        assert_eq!(get_state(&client).await, "expanded");
    });
}

/// RAII guard to kill the dbus-daemon child when the test ends.
struct BusGuard(Option<std::process::Child>);

impl Drop for BusGuard {
    fn drop(&mut self) {
        if let Some(mut child) = self.0.take() {
            let _ = child.kill();
        }
    }
}
