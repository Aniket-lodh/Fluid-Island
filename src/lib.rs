//! Fluid Island core library.
//!
//! Public API for the Rust core: state machine, IPC/D-Bus service, and UI.

pub mod app;
pub mod error;
pub mod integrations;
pub mod ipc;
pub mod state;
pub mod ui;

pub use ipc::{start_ipc_service, FluidIslandInterface, IpcEvent, SharedState};
pub use state::IslandState;
