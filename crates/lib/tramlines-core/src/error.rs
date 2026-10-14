//! The engine's one error type: a grammar the engine carries that does not
//! read, which is a defect of the build, never of a user's input.

/// Convenience alias for results from this crate.
pub type Result<T> = std::result::Result<T, Error>;

// sokf:begin error
/// Errors produced by this crate.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// A broken invariant.
    #[error("internal: {0}")]
    Internal(String),
}
// sokf:end error
