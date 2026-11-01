mod app;
mod error;
mod integrations;
mod ipc;
mod state;
mod ui;

fn main() {
    tracing_subscriber::fmt::init();
    let application = app::FluidApplication::new();
    application.run();
}
