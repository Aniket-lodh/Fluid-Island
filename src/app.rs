use crate::state::IslandState;
use crate::ui::FallbackWindow;
use libadwaita as adw;
use libadwaita::gio::prelude::{ApplicationExt, ApplicationExtManual};
use std::cell::{Cell, RefCell};
use std::rc::Rc;

/// The Fluid Island GTK application.
#[allow(dead_code)]
pub struct FluidApplication {
    application: adw::Application,
    state: Rc<Cell<IslandState>>,
    window: Rc<RefCell<Option<FallbackWindow>>>,
}

impl FluidApplication {
    /// Create the application object and wire the fallback window.
    pub fn new() -> Self {
        let application = adw::Application::new(Some("org.fluidisland.Core"), Default::default());

        let state = Rc::new(Cell::new(IslandState::Collapsed));
        let window_slot = Rc::new(RefCell::new(None));

        let state_for_activate = state.clone();
        let slot_for_activate = window_slot.clone();
        application.connect_activate(move |gtk_application| {
            let fallback = FallbackWindow::new(gtk_application, state_for_activate.clone());
            fallback.present();
            *slot_for_activate.borrow_mut() = Some(fallback);
        });

        Self {
            application,
            state,
            window: window_slot,
        }
    }

    /// Run the application's main loop.
    pub fn run(&self) {
        self.application.run();
    }
}
