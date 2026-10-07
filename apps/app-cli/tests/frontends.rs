#![allow(clippy::unwrap_used)]

use std::io::Write;
use std::process::{Command, Stdio};

use serde_json::{Value, json};

fn run(args: &[&str]) -> Value {
    let output = Command::new(env!("CARGO_BIN_EXE_app-cli")).args(args).output().unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    serde_json::from_slice(&output.stdout).unwrap()
}

#[test]
fn commands_and_execution_share_engine() {
    let commands = run(&["commands"]);
    assert!(commands.as_array().unwrap().iter().any(|item| item["id"] == "demo.incrementCounter"));
    let result = run(&["run", "demo.incrementCounter"]);
    assert_eq!(result["state"]["model"]["counter"], 1);
    let result = run(&["run", "--cmd", "demo.incrementCounter={\"amount\":5}", "--cmd", "edit.undo={}", "--cmd", "edit.redo={}"]);
    assert_eq!(result["state"]["model"]["counter"], 5);
}

#[test]
fn native_storage_round_trip() {
    let path = std::env::temp_dir().join(format!("app-cli-test-{}.json", std::process::id()));
    let path_text = path.to_str().unwrap();
    run(&["run", "demo.setName", "{\"name\":\"Saved\"}", "--save", path_text]);
    let state = run(&["state", "--load", path_text]);
    assert_eq!(state["model"]["name"], "Saved");
    std::fs::remove_file(path).unwrap();
}

#[test]
fn invalid_input_exits_nonzero() {
    for args in [vec!["run", "missing"], vec!["run", "demo.setName", "[]"], vec!["run"], vec!["mcp", "--save", "ignored.json"]] {
        let output = Command::new(env!("CARGO_BIN_EXE_app-cli")).args(args).output().unwrap();
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
        assert!(!output.stderr.is_empty());
    }
}

#[test]
fn stdio_mcp_lists_runs_and_inspects_state() {
    let mut child =
        Command::new(env!("CARGO_BIN_EXE_app-cli")).arg("mcp").stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().unwrap();
    let messages = [
        json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{
            "protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"test","version":"1"}}}),
        json!({"jsonrpc":"2.0","method":"notifications/initialized"}),
        json!({"jsonrpc":"2.0","id":2,"method":"tools/list"}),
        json!({"jsonrpc":"2.0","id":3,"method":"tools/call","params":{
            "name":"command_run","arguments":{"command":"demo.incrementCounter"}}}),
        json!({"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"state_get"}}),
    ];
    let mut stdin = child.stdin.take().unwrap();
    for message in messages {
        writeln!(stdin, "{message}").unwrap();
    }
    drop(stdin);
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let replies: Vec<Value> = String::from_utf8(output.stdout).unwrap().lines().map(|line| serde_json::from_str(line).unwrap()).collect();
    assert_eq!(replies.len(), 4);
    assert_eq!(replies[1]["result"]["tools"].as_array().unwrap().len(), 3);
    assert_eq!(replies[3]["result"]["structuredContent"]["result"]["model"]["counter"], 1);
}

#[cfg(unix)]
#[test]
fn non_utf8_arguments_return_errors_without_panicking() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;
    let output = Command::new(env!("CARGO_BIN_EXE_app-cli")).arg(OsString::from_vec(vec![255])).output().unwrap();
    assert!(!output.status.success());
    let error = String::from_utf8(output.stderr).unwrap();
    assert!(error.contains("arguments must be UTF-8"));
    assert!(!error.contains("panicked"));
}
