use crate::ipc::{start_ipc_service, IpcEvent, SharedState};
use crate::state::IslandState;
use crate::stopwatch::Stopwatch;
use crate::ui::FallbackWindow;
use gtk4::glib;
use libadwaita as adw;
use libadwaita::gio::prelude::{ApplicationExt, ApplicationExtManual};
use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

thread_local! {
    static FALLBACK_WINDOW: RefCell<Option<FallbackWindow>> =
        const { RefCell::new(None) };
}

fn fallback_enabled() -> bool {
    std::env::var("FLUID_ISLAND_FALLBACK").is_ok_and(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        || std::env::args().any(|a| a == "--fallback")
}

/// Headless by default; the GTK window only appears with `FLUID_ISLAND_FALLBACK=1` or `--fallback`.
// Fields are retained for ownership; state is mutated through shared Arcs.
#[allow(dead_code)]
pub struct FluidApplication {
    application: adw::Application,
    state: Arc<Mutex<IslandState>>,
    attached: Arc<AtomicBool>,
    _ipc_service: crate::ipc::service::IpcServiceHandle,
    _hold_guard: Option<libadwaita::gio::ApplicationHoldGuard>,
}

#[allow(clippy::new_without_default)]
impl FluidApplication {
    /// The fallback window is only created when fallback mode is enabled.
    pub fn new() -> Self {
        let fallback = fallback_enabled();
        let flags = if fallback {
            libadwaita::gio::ApplicationFlags::empty()
        } else {
            libadwaita::gio::ApplicationFlags::IS_SERVICE
        };
        let application = adw::Application::new(None, flags);
        let hold_guard = if fallback {
            None
        } else {
            Some(application.hold())
        };

        let state = Arc::new(Mutex::new(IslandState::Collapsed));
        let attached = Arc::new(AtomicBool::new(false));
        let stopwatch = Arc::new(Mutex::new(Stopwatch::new()));

        let shared = SharedState {
            state: state.clone(),
            attached: attached.clone(),
            stopwatch: stopwatch.clone(),
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
        let stopwatch_for_activate = stopwatch.clone();
        application.connect_activate(move |gtk_application| {
            if !fallback_enabled() {
                // Headless shell-owned mode: no GTK window, not an Alt+Tab app.
                return;
            }
            let fallback = FallbackWindow::new(
                gtk_application,
                state_for_activate.clone(),
                stopwatch_for_activate.clone(),
            );

            // The window owns its widgets; store it, then operate through the slot.
            FALLBACK_WINDOW.with(|slot| {
                *slot.borrow_mut() = Some(fallback);
            });
            FALLBACK_WINDOW.with(|slot| {
                if let Some(window) = slot.borrow().as_ref() {
                    if attached_for_activate.load(Ordering::SeqCst) {
                        window.hide();
                    } else {
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
            _hold_guard: hold_guard,
        }
    }

    pub fn run(&self) {
        // Filter --fallback so GApplication doesn't see an unknown option.
        let args: Vec<String> = std::env::args().filter(|a| a != "--fallback").collect();
        self.application.run_with_args(&args);
    }
}
