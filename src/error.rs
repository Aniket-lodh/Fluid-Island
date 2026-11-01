use thiserror::Error;

/// Errors produced by the Fluid Island core.
#[allow(dead_code)]
#[derive(Debug, Error)]
pub enum FluidError {
    /// Placeholder for application-level failures; expanded in later milestones.
    #[error("application error")]
    Application,
}
