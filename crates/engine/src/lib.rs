//! UI-independent command execution, history, events, and inspection.
#![forbid(unsafe_code)]

mod commands;
mod input;
mod session;

pub use app_core::AppError;
pub use commands::{CommandRegistry, CommandSpec};
pub use session::{CommandResult, Event, Session};

#[cfg(test)]
mod tests;
