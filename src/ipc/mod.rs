//! Inter-process communication (D-Bus).

pub mod service;

#[allow(unused_imports)]
pub use service::{start_ipc_service, FluidIslandInterface, SharedState};

/// Events sent from the IPC thread to the GTK main thread.
#[derive(Clone, Copy, Debug)]
pub enum IpcEvent {
    StateChanged(crate::state::IslandState),
    AttachmentChanged(bool),
}
