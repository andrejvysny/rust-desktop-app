//! A small MCP frontend for the shared command engine.
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

mod server;
mod tools;
pub use server::{PROTOCOL_VERSION, Server};
pub use tools::command_list;

#[cfg(test)]
mod tests;
