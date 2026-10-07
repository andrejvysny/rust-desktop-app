use std::io::{BufRead, Write};

use app_core::MAX_INPUT_BYTES;
use app_platform::CommandHost;
use serde_json::{Value, json};

use crate::tools;

pub const PROTOCOL_VERSION: &str = "2025-06-18";
const VERSIONS: &[&str] = &[PROTOCOL_VERSION];
type RpcResult = Result<Value, (i64, String)>;
type Frame = Result<Vec<u8>, ()>;

/// One persistent command host per connection.
pub struct Server {
    host: CommandHost,
    negotiated: bool,
    initialized: bool,
}

impl Server {
    pub fn new(host: CommandHost) -> Self {
        Self { host, negotiated: false, initialized: false }
    }

    pub fn serve(&mut self, mut input: impl BufRead, mut output: impl Write) -> std::io::Result<()> {
        loop {
            let Some(line) = bounded_line(&mut input)? else { return Ok(()) };
            let reply = match line {
                Ok(bytes) => match std::str::from_utf8(&bytes) {
                    Ok(text) => self.handle_line(text),
                    Err(_) => Some(error(Value::Null, -32700, "invalid UTF-8").to_string()),
                },
                Err(()) => Some(error(Value::Null, -32600, "input exceeds byte limit").to_string()),
            };
            if let Some(reply) = reply {
                writeln!(output, "{reply}")?;
                output.flush()?;
            }
        }
    }

    pub fn handle_line(&mut self, line: &str) -> Option<String> {
        if line.len() > MAX_INPUT_BYTES {
            return Some(error(Value::Null, -32600, "input exceeds byte limit").to_string());
        }
        if line.trim().is_empty() {
            return None;
        }
        match serde_json::from_str(line) {
            Ok(value) => self.handle(value).map(|reply| reply.to_string()),
            Err(_) => Some(error(Value::Null, -32700, "invalid JSON").to_string()),
        }
    }

    pub fn handle(&mut self, message: Value) -> Option<Value> {
        let Some(object) = message.as_object() else {
            return Some(error(Value::Null, -32600, "request must be an object"));
        };
        let id = object.get("id").cloned();
        if object.get("jsonrpc").and_then(Value::as_str) != Some("2.0") {
            return Some(error(valid_id(id.as_ref()), -32600, "jsonrpc must be 2.0"));
        }
        let Some(method) = object.get("method").and_then(Value::as_str) else {
            return Some(error(valid_id(id.as_ref()), -32600, "method must be a string"));
        };
        if id.as_ref().is_some_and(|id| !is_id(id)) {
            return Some(error(Value::Null, -32600, "id must be a string or integer"));
        }
        let params = object.get("params").cloned().unwrap_or_else(|| json!({}));
        let Some(id) = id else {
            if method == "notifications/initialized" && self.negotiated && params.is_object() {
                self.initialized = true;
            }
            return None;
        };
        Some(match self.request(method, &params) {
            Ok(result) => json!({"jsonrpc":"2.0","id":id,"result":result}),
            Err((code, message)) => error(id, code, &message),
        })
    }

    fn request(&mut self, method: &str, params: &Value) -> RpcResult {
        if !params.is_object() {
            return Err((-32602, "params must be an object".into()));
        }
        match method {
            "initialize" => self.initialize(params),
            "ping" => Ok(json!({})),
            _ if !self.initialized => Err((-32002, "server not initialized".into())),
            "tools/list" => Ok(json!({"tools":tools::definitions()})),
            "tools/call" => self.call(params),
            _ => Err((-32601, format!("method not found: {method}"))),
        }
    }

    fn initialize(&mut self, params: &Value) -> RpcResult {
        if self.negotiated {
            return Err((-32600, "already initialized".into()));
        }
        let version = params.get("protocolVersion").and_then(Value::as_str).ok_or((-32602, "protocolVersion must be a string".into()))?;
        if !params.get("capabilities").is_some_and(Value::is_object)
            || !params
                .get("clientInfo")
                .is_some_and(|client| client.get("name").is_some_and(Value::is_string) && client.get("version").is_some_and(Value::is_string))
        {
            return Err((-32602, "capabilities and clientInfo are required".into()));
        }
        let version = if VERSIONS.contains(&version) { version } else { PROTOCOL_VERSION };
        self.negotiated = true;
        Ok(json!({"protocolVersion":version,"capabilities":{"tools":{}},
            "serverInfo":{"name":"app-template","version":env!("CARGO_PKG_VERSION")},
            "instructions":"List commands, run commands, then inspect state."}))
    }

    fn call(&mut self, params: &Value) -> RpcResult {
        let name = params.get("name").and_then(Value::as_str).ok_or((-32602, "name must be a string".into()))?;
        if !["command_list", "command_run", "state_get"].contains(&name) {
            return Err((-32602, format!("unknown tool: {name}")));
        }
        let arguments = params.get("arguments").cloned().unwrap_or_else(|| json!({}));
        tools::validate(name, &arguments).map_err(|message| (-32602, message))?;
        match tools::call(&mut self.host, name, &arguments) {
            Ok(value) => Ok(json!({"content":[{"type":"text","text":value.to_string()}],
                "structuredContent":{"result":value},"isError":false})),
            Err(message) => Ok(json!({"content":[{"type":"text","text":message}],"isError":true})),
        }
    }
}

fn is_id(id: &Value) -> bool {
    id.is_string() || id.is_i64() || id.is_u64()
}
fn valid_id(id: Option<&Value>) -> Value {
    id.filter(|value| is_id(value)).cloned().unwrap_or(Value::Null)
}
fn error(id: Value, code: i64, message: &str) -> Value {
    json!({"jsonrpc":"2.0","id":id,"error":{"code":code,"message":message}})
}

// Drain oversized frames without retaining their contents, then recover at the next newline.
fn bounded_line(input: &mut impl BufRead) -> std::io::Result<Option<Frame>> {
    let mut line = Vec::new();
    let mut oversized = false;
    let mut consumed = false;
    loop {
        let buffer = input.fill_buf()?;
        if buffer.is_empty() {
            return Ok(consumed.then_some(if oversized { Err(()) } else { Ok(line) }));
        }
        consumed = true;
        let newline = buffer.iter().position(|byte| *byte == b'\n');
        let count = newline.map_or(buffer.len(), |index| index + 1);
        let content_count = newline.unwrap_or(count);
        if line.len().saturating_add(content_count) > MAX_INPUT_BYTES {
            oversized = true;
            line.clear();
        } else if !oversized {
            let content =
                buffer.get(..content_count).ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid frame boundary"))?;
            line.extend_from_slice(content);
        }
        input.consume(count);
        if newline.is_some() {
            return Ok(Some(if oversized { Err(()) } else { Ok(line) }));
        }
    }
}
