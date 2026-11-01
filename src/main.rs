mod app;
mod error;
mod integrations;
mod ipc;
mod state;
mod ui;

fn main() {
    let application = app::FluidApplication::new();
    application.run();
}
