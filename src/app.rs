use libadwaita as adw;
use libadwaita::gio::prelude::{ApplicationExt, ApplicationExtManual};

/// The Fluid Island GTK application.
pub struct FluidApplication {
    application: adw::Application,
}

impl FluidApplication {
    /// Create the application object. UI wiring is added in later milestones.
    pub fn new() -> Self {
        let application = adw::Application::new(Some("org.fluidisland.Core"), Default::default());

        application.connect_activate(|_application| {
            // The fallback window and state wiring are implemented in later
            // milestones. Intentionally empty for the scaffold.
        });

        Self { application }
    }

    /// Run the application's main loop.
    pub fn run(&self) {
        self.application.run();
    }
}
