//! D-Bus service implementation.

use crate::ipc::IpcEvent;
use crate::state::IslandState;
use crate::stopwatch::{format_duration_cs, Stopwatch};
use crate::system_info;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::thread;
use tracing::{error, info};

/// Shared state between the GTK thread and the IPC thread.
#[derive(Clone)]
pub struct SharedState {
    pub state: Arc<Mutex<IslandState>>,
    pub attached: Arc<AtomicBool>,
    pub stopwatch: Arc<Mutex<Stopwatch>>,
}

/// Owns the D-Bus service thread. Keeping the handle alive keeps the IPC
/// service thread running.
#[allow(dead_code)]
pub struct IpcServiceHandle {
    pub thread: thread::JoinHandle<()>,
}

pub fn start_ipc_service(
    shared: SharedState,
    on_event: Arc<dyn Fn(IpcEvent) + Send + Sync>,
) -> IpcServiceHandle {
    let thread = thread::spawn(move || {
        run_service(shared, on_event);
    });
    IpcServiceHandle { thread }
}

fn run_service(shared: SharedState, on_event: Arc<dyn Fn(IpcEvent) + Send + Sync>) {
    zbus::block_on(async {
        let connection = match zbus::Connection::session().await {
            Ok(connection) => connection,
            Err(err) => {
                error!("failed to connect to the session bus: {err}");
                return;
            }
        };

        let interface = FluidIslandInterface {
            shared: shared.clone(),
            on_event: on_event.clone(),
        };

        if let Err(err) = connection
            .object_server()
            .at("/org/fluidisland/Core", interface)
            .await
        {
            error!("failed to register the D-Bus object: {err}");
            return;
        }

        if let Err(err) = connection.request_name("org.fluidisland.Core").await {
            error!("failed to acquire the D-Bus name: {err}");
            return;
        }

        info!("Fluid Island D-Bus service is running");
        connection.closed().await;
        info!("Fluid Island D-Bus connection closed");
    });
}

#[derive(Clone)]
pub struct FluidIslandInterface {
    pub shared: SharedState,
    pub on_event: Arc<dyn Fn(IpcEvent) + Send + Sync>,
}

#[zbus::interface(name = "org.fluidisland.Core")]
impl FluidIslandInterface {
    fn get_state(&self) -> String {
        let state = self
            .shared
            .state
            .lock()
            .map(|guard| *guard)
            .unwrap_or_default();
        state.as_str().to_owned()
    }

    async fn toggle(
        &mut self,
        #[zbus(signal_emitter)] emitter: zbus::object_server::SignalEmitter<'_>,
    ) -> String {
        let new_state = {
            let mut guard = self
                .shared
                .state
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            guard.toggle();
            *guard
        };

        (self.on_event)(IpcEvent::StateChanged(new_state));

        let _ = emitter.state_changed(new_state.as_str()).await;
        new_state.as_str().to_owned()
    }

    async fn attach(
        &mut self,
        #[zbus(signal_emitter)] emitter: zbus::object_server::SignalEmitter<'_>,
    ) -> String {
        self.shared.attached.store(true, Ordering::SeqCst);

        let state = self
            .shared
            .state
            .lock()
            .map(|guard| *guard)
            .unwrap_or_default();

        (self.on_event)(IpcEvent::AttachmentChanged(true));
        let _ = emitter.state_changed(state.as_str()).await;

        state.as_str().to_owned()
    }

    async fn detach(&mut self) {
        self.shared.attached.store(false, Ordering::SeqCst);
        (self.on_event)(IpcEvent::AttachmentChanged(false));
    }

    fn get_system_info(&self) -> (String, String, String, String, String) {
        let (time, tz) = system_info::current_time();
        let uptime = system_info::read_uptime().unwrap_or_default();
        let memory = system_info::read_memory_usage().unwrap_or_default();
        let load = system_info::read_loadavg().unwrap_or_default();
        (time, tz.unwrap_or_default(), uptime, memory, load)
    }

    fn get_stopwatch(&self) -> (String, String) {
        let sw = self
            .shared
            .stopwatch
            .lock()
            .map(|g| g.clone())
            .unwrap_or_default();
        let elapsed = sw.elapsed();
        let status = if sw.is_running() {
            "running"
        } else if elapsed.is_zero() {
            "stopped"
        } else {
            "paused"
        };
        (format_duration_cs(elapsed), status.to_string())
    }

    fn get_laps(&self) -> Vec<String> {
        self.shared
            .stopwatch
            .lock()
            .map(|g| g.laps().iter().map(|d| format_duration_cs(*d)).collect())
            .unwrap_or_default()
    }

    fn stopwatch_lap(&self) {
        let mut sw = self
            .shared
            .stopwatch
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        sw.lap();
    }

    fn stopwatch_start(&self) {
        let mut sw = self
            .shared
            .stopwatch
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        sw.start();
    }

    fn stopwatch_pause(&self) {
        let mut sw = self
            .shared
            .stopwatch
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        sw.pause();
    }

    fn stopwatch_resume(&self) {
        let mut sw = self
            .shared
            .stopwatch
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        sw.resume();
    }

    fn stopwatch_reset(&self) {
        let mut sw = self
            .shared
            .stopwatch
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        sw.reset();
    }

    #[zbus(signal)]
    async fn state_changed(
        emitter: &zbus::object_server::SignalEmitter<'_>,
        state: &str,
    ) -> zbus::Result<()> {
        // The macro generates the signal from this declaration.
        let _ = (emitter, state);
        Ok(())
    }
}
