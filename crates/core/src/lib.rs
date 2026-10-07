//! Shared errors and input limits without UI or platform dependencies.
#![forbid(unsafe_code)]

/// Maximum encoded JSON input size accepted by the engine.
pub const MAX_INPUT_BYTES: usize = 1024 * 1024;
/// Maximum UTF-8 byte length of the demonstration name.
pub const MAX_NAME_BYTES: usize = 256;
/// Maximum number of retained undo snapshots.
pub const MAX_HISTORY: usize = 100;

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum AppError {
    #[error("invalid parameters: {0}")]
    InvalidParams(String),
    #[error("unknown command: {0}")]
    UnknownCommand(String),
    #[error("disabled command: {0}")]
    DisabledCommand(String),
    #[error("limit exceeded: {0}")]
    LimitExceeded(String),
    #[error("storage error: {0}")]
    Storage(String),
    #[error("serialization error: {0}")]
    Serialization(String),
}
