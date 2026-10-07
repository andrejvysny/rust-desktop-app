use std::io::Cursor;

use app_core::MAX_INPUT_BYTES;
use app_platform::CommandHost;
use serde_json::{Value, json};

use crate::Server;

fn request(server: &mut Server, id: i64, method: &str, params: Value) -> Value {
    server.handle(json!({"jsonrpc":"2.0","id":id,"method":method,"params":params})).unwrap()
}

fn initialized() -> Server {
    let mut server = Server::new(CommandHost::default());
    let reply = request(
        &mut server,
        1,
        "initialize",
        json!({
            "protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"test","version":"1"}
        }),
    );
    assert_eq!(reply["result"]["protocolVersion"], "2025-06-18");
    assert!(server.handle(json!({"jsonrpc":"2.0","method":"notifications/initialized"})).is_none());
    server
}

#[test]
fn lifecycle_and_persistent_tools() {
    let mut server = initialized();
    let reply = request(&mut server, 2, "tools/list", json!({}));
    assert_eq!(reply["result"]["tools"].as_array().unwrap().len(), 3);
    let commands = request(&mut server, 3, "tools/call", json!({"name":"command_list"}));
    assert!(commands["result"]["structuredContent"]["result"].as_array().unwrap().iter().any(|command| command["id"] == "demo.incrementCounter"));
    let changed = request(
        &mut server,
        4,
        "tools/call",
        json!({
            "name":"command_run","arguments":{"command":"demo.incrementCounter","params":{}}
        }),
    );
    assert_eq!(changed["result"]["isError"], false);
    let state = request(&mut server, 5, "tools/call", json!({"name":"state_get"}));
    assert_eq!(state["result"]["structuredContent"]["result"]["model"]["counter"], 1);
    let undone = request(
        &mut server,
        6,
        "tools/call",
        json!({
            "name":"command_run","arguments":{"command":"edit.undo"}
        }),
    );
    assert_eq!(undone["result"]["isError"], false);
    let state = request(&mut server, 7, "tools/call", json!({"name":"state_get"}));
    assert_eq!(state["result"]["structuredContent"]["result"]["model"]["counter"], 0);
}

#[test]
fn invalid_requests_and_command_errors() {
    let mut server = initialized();
    for message in [
        json!([]),
        json!({"id":1,"method":"ping"}),
        json!({"jsonrpc":"2.0","id":true,"method":"ping"}),
        json!({"jsonrpc":"2.0","id":1.5,"method":"ping"}),
    ] {
        assert_eq!(server.handle(message).unwrap()["error"]["code"], -32600);
    }
    assert_eq!(request(&mut server, 1, "tools/call", json!([]))["error"]["code"], -32602);
    let failed = request(
        &mut server,
        2,
        "tools/call",
        json!({
            "name":"command_run","arguments":{"command":"missing"}
        }),
    );
    assert_eq!(failed["result"]["isError"], true);
    assert!(server.handle(json!({"jsonrpc":"2.0","method":"unknown"})).is_none());
    assert_eq!(serde_json::from_str::<Value>(&server.handle_line("{").unwrap()).unwrap()["error"]["code"], -32700);
}

#[test]
fn initialize_requires_valid_metadata_and_lifecycle() {
    let mut server = Server::new(CommandHost::default());
    assert_eq!(request(&mut server, 1, "tools/list", json!({}))["error"]["code"], -32002);
    assert_eq!(request(&mut server, 2, "initialize", json!({"protocolVersion":1}))["error"]["code"], -32602);
    let reply = request(
        &mut server,
        3,
        "initialize",
        json!({
            "protocolVersion":"future","capabilities":{},"clientInfo":{"name":"test","version":"1"}
        }),
    );
    assert_eq!(reply["result"]["protocolVersion"], "2025-06-18");
    assert_eq!(request(&mut server, 4, "tools/list", json!({}))["error"]["code"], -32002);
    assert!(server.handle(json!({"jsonrpc":"2.0","method":"notifications/initialized"})).is_none());
    assert!(request(&mut server, 5, "tools/list", json!({}))["result"].is_object());
}

#[test]
fn oversized_and_non_utf8_frames_recover() {
    let mut server = initialized();
    let mut input = vec![b'x'; MAX_INPUT_BYTES + 1];
    input.extend_from_slice(b"\n\xff\n{\"jsonrpc\":\"2.0\",\"id\":9,\"method\":\"ping\"}\n");
    let mut output = Vec::new();
    server.serve(Cursor::new(input), &mut output).unwrap();
    let replies: Vec<Value> = String::from_utf8(output).unwrap().lines().map(|line| serde_json::from_str(line).unwrap()).collect();
    assert_eq!(replies.len(), 3);
    assert_eq!(replies[0]["error"]["code"], -32600);
    assert_eq!(replies[1]["error"]["code"], -32700);
    assert_eq!(replies[2]["id"], 9);
}

#[test]
fn mcp_matches_headless_acceptance_helper_and_rejects_hostile_params() {
    let expected = app_testkit::run_commands(&[("demo.incrementCounter", json!({"amount": 3}))]).unwrap();
    let mut server = initialized();
    request(
        &mut server,
        10,
        "tools/call",
        json!({"name":"command_run",
        "arguments":{"command":"demo.incrementCounter","params":{"amount":3}}}),
    );
    let state = request(&mut server, 11, "tools/call", json!({"name":"state_get"}));
    assert_eq!(state["result"]["structuredContent"]["result"], expected.session().inspect());
    for params in app_testkit::hostile_params() {
        let reply = request(
            &mut server,
            12,
            "tools/call",
            json!({"name":"command_run",
            "arguments":{"command":"demo.incrementCounter","params":params}}),
        );
        assert!(reply["error"].is_object() || reply["result"]["isError"] == true);
    }
}

#[test]
fn discovery_and_path_storage_use_shared_host() {
    let mut server = initialized();
    let discovery = request(&mut server, 20, "tools/call", json!({"name":"command_list"}));
    let commands = discovery["result"]["structuredContent"]["result"].as_array().unwrap();
    for id in ["app.open", "app.save"] {
        let command = commands.iter().find(|command| command["id"] == id).unwrap();
        assert_eq!(command["params"]["oneOf"][1]["required"], json!(["path"]));
    }
    for (number, command, params) in [
        (21, "demo.incrementCounter", json!({"amount":4})),
        (22, "app.save", json!({"path":"state.json"})),
        (23, "app.new", json!({})),
        (24, "app.open", json!({"path":"state.json"})),
    ] {
        let reply = request(
            &mut server,
            number,
            "tools/call",
            json!({"name":"command_run",
            "arguments":{"command":command,"params":params}}),
        );
        assert_eq!(reply["result"]["isError"], false);
    }
    let state = request(&mut server, 25, "tools/call", json!({"name":"state_get"}));
    assert_eq!(state["result"]["structuredContent"]["result"]["model"]["counter"], 4);
    assert_eq!(state["result"]["structuredContent"]["result"]["can_undo"], false);
}
