use thiserror::Error;

/// Errors produced by the Fluid Island core; variants added as integrations land.
#[allow(dead_code)]
#[derive(Debug, Error)]
pub enum FluidError {
    #[error("application error")]
    Application,
}
