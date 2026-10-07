//! Headless command and MCP frontend.
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

mod options;

use std::io::{self, Write};
use std::process::ExitCode;

use app_core::{AppError, MAX_INPUT_BYTES};
use app_platform::{CommandHost, NativeStorage};
use serde_json::{Value, json};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), AppError> {
    let options = options::Options::parse(arguments()?.into_iter())?;
    if options.action == "help" {
        println!("app-cli commands | state | run [COMMAND [JSON]] [--cmd ID=JSON] [--load PATH] [--save PATH] | mcp");
        return Ok(());
    }
    let mut host = CommandHost::new(Box::new(NativeStorage));
    if options.action == "mcp" {
        return app_mcp::Server::new(host).serve(io::stdin().lock(), io::stdout().lock()).map_err(|error| AppError::Storage(error.to_string()));
    }
    if let Some(path) = options.load {
        host.execute("app.open", json!({"path":path}))?;
    }
    let mut result = Value::Null;
    for (command, params) in options.commands {
        result = serde_json::to_value(host.execute(&command, params)?).map_err(|error| AppError::Serialization(error.to_string()))?;
    }
    if let Some(path) = options.save {
        host.execute("app.save", json!({"path":path}))?;
    }
    let value = match options.action.as_str() {
        "commands" => app_mcp::command_list(&host),
        "state" => host.session().inspect(),
        _ => result,
    };
    writeln!(io::stdout().lock(), "{value}").map_err(|error| AppError::Storage(error.to_string()))
}

fn arguments() -> Result<Vec<String>, AppError> {
    let mut arguments = Vec::new();
    let mut bytes = 0usize;
    for argument in std::env::args_os().skip(1) {
        let argument = argument.into_string().map_err(|_| AppError::InvalidParams("arguments must be UTF-8".into()))?;
        bytes = bytes.saturating_add(argument.len());
        if bytes > MAX_INPUT_BYTES || arguments.len() >= 4096 {
            return Err(AppError::LimitExceeded("arguments exceed input limit".into()));
        }
        arguments.push(argument);
    }
    Ok(arguments)
}
