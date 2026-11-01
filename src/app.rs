use crate::ipc::{start_ipc_service, IpcEvent, SharedState};
use crate::state::IslandState;
use crate::ui::FallbackWindow;
use gtk4::glib;
use libadwaita as adw;
use libadwaita::gio::prelude::{ApplicationExt, ApplicationExtManual};
use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

thread_local! {
    static FALLBACK_WINDOW: RefCell<Option<FallbackWindow>> =
        const { RefCell::new(None) };
}

/// The Fluid Island GTK application.
#[allow(dead_code)]
pub struct FluidApplication {
    application: adw::Application,
    state: Arc<Mutex<IslandState>>,
    attached: Arc<AtomicBool>,
    _ipc_service: crate::ipc::service::IpcServiceHandle,
}

#[allow(clippy::new_without_default)]
impl FluidApplication {
    /// Create the application object, wire the IPC service, and prepare the
    /// fallback window. The window is shown on activation unless the extension
    /// attaches within the startup grace period.
    pub fn new() -> Self {
        // Use None for the GTK application id so GApplication does not try to
        // own the session-bus name; we manage org.fluidisland.Core ourselves
        // via zbus. A shared id would conflict with GTK's own registration.
        let application = adw::Application::new(None, Default::default());

        let state = Arc::new(Mutex::new(IslandState::Collapsed));
        let attached = Arc::new(AtomicBool::new(false));

        let shared = SharedState {
            state: state.clone(),
            attached: attached.clone(),
        };

        let on_event = Arc::new(move |event: IpcEvent| {
            glib::MainContext::default().invoke(move || {
                FALLBACK_WINDOW.with(|slot| {
                    if let Some(window) = slot.borrow().as_ref() {
                        match event {
                            IpcEvent::StateChanged(state) => window.set_state(state),
                            IpcEvent::AttachmentChanged(true) => window.hide(),
                            IpcEvent::AttachmentChanged(false) => window.present(),
                        }
                    }
                });
            });
        });

        let ipc_service = start_ipc_service(shared, on_event);

        let state_for_activate = state.clone();
        let attached_for_activate = attached.clone();
        let attached_for_struct = attached.clone();
        application.connect_activate(move |gtk_application| {
            let fallback = FallbackWindow::new(gtk_application, state_for_activate.clone());

            FALLBACK_WINDOW.with(|slot| {
                *slot.borrow_mut() = Some(fallback.clone());
            });

            // If the extension is already attached, keep the fallback hidden
            // and skip the grace timeout entirely.
            if attached_for_activate.load(Ordering::SeqCst) {
                fallback.hide();
                return;
            }

            // Startup grace period: keep the fallback hidden briefly so an
            // attaching extension can take over without a visible flash. If the
            // extension does not attach within the grace period, show the
            // fallback window so the application remains usable standalone.
            let fallback_for_timeout = FALLBACK_WINDOW.with(|slot| slot.borrow().clone());
            let attached_for_timeout = attached.clone();
            glib::timeout_add_local_once(Duration::from_millis(250), move || {
                if !attached_for_timeout.load(Ordering::SeqCst) {
                    if let Some(window) = fallback_for_timeout {
                        window.present();
                    }
                }
            });
        });

        Self {
            application,
            state,
            attached: attached_for_struct,
            _ipc_service: ipc_service,
        }
    }

    /// Run the application's main loop.
    pub fn run(&self) {
        self.application.run();
    }
}
