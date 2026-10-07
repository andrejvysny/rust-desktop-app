//! Portable workspace quality gates: `cargo xtask <command>`.
mod assets;
mod layers;

use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    let result = match std::env::args().nth(1).as_deref() {
        Some("ci") => ci(),
        Some("layers") => metadata().and_then(|data| layers::run(&data)),
        Some("assets") => assets::run(&root()),
        Some("wasm") => wasm(),
        None | Some("help" | "--help" | "-h") => {
            println!("usage: cargo xtask <ci|layers|assets|wasm>");
            Ok(())
        }
        Some(command) => Err(format!("unknown xtask command: {command}")),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn cargo() -> Command {
    let mut command = Command::new(std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into()));
    command.current_dir(root());
    command
}

fn run(args: &[&str]) -> Result<(), String> {
    eprintln!("$ cargo {}", args.join(" "));
    let status = cargo().args(args).status().map_err(|error| format!("cargo: {error}"))?;
    if status.success() { Ok(()) } else { Err(format!("cargo {} failed: {status}", args.join(" "))) }
}

fn metadata() -> Result<serde_json::Value, String> {
    let output = cargo().args(["metadata", "--format-version", "1", "--no-deps"]).output().map_err(|error| format!("cargo metadata: {error}"))?;
    if !output.status.success() {
        return Err(format!("cargo metadata failed: {}", String::from_utf8_lossy(&output.stderr)));
    }
    serde_json::from_slice(&output.stdout).map_err(|error| format!("cargo metadata: {error}"))
}

fn wasm() -> Result<(), String> {
    let output = Command::new("rustup")
        .args(["target", "list", "--installed"])
        .output()
        .map_err(|error| format!("rustup: {error}; install wasm32-unknown-unknown with rustup target add"))?;
    if !output.status.success() || !String::from_utf8_lossy(&output.stdout).lines().any(|target| target == "wasm32-unknown-unknown") {
        return Err("missing WASM target; run: rustup target add wasm32-unknown-unknown".into());
    }
    run(&[
        "check",
        "--target",
        "wasm32-unknown-unknown",
        "-p",
        "app-core",
        "-p",
        "app-model",
        "-p",
        "app-engine",
        "-p",
        "app-platform",
        "-p",
        "app-ui-egui",
        "-p",
        "app-testkit",
        "-p",
        "app-mcp",
    ])?;
    run(&["build", "--target", "wasm32-unknown-unknown", "-p", "app-web"])
}

fn ci() -> Result<(), String> {
    run(&["fmt", "--all", "--check"])?;
    run(&["clippy", "--workspace", "--all-targets", "--", "-D", "warnings"])?;
    run(&["test", "--workspace"])?;
    assets::run(&root())?;
    layers::run(&metadata()?)?;
    wasm()?;
    println!("ci: all checks passed");
    Ok(())
}
