use thiserror::Error;

/// Diagnostic failure. Library paths do not clamp a rejected input into range.
#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Config(String),
    #[error("overflow: {0}")]
    Overflow(String),
    #[error("snapshot: {0}")]
    Snapshot(String),
    #[error("invariant: {0}")]
    Invariant(String),
    #[error("run: {0}")]
    Run(String),
    #[error("io: {0}")]
    Io(String),
}
