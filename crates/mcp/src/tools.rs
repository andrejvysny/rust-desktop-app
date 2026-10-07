use app_platform::CommandHost;
use serde_json::{Value, json};

pub(crate) fn definitions() -> Value {
    json!([
        {"name":"command_list","description":"List commands and parameter schemas.",
         "inputSchema":{"type":"object","properties":{},"additionalProperties":false}},
        {"name":"command_run","description":"Run an application command.",
         "inputSchema":{"type":"object","properties":{"command":{"type":"string"},
             "params":{"type":"object"}},"required":["command"],"additionalProperties":false}},
        {"name":"state_get","description":"Inspect application state and undo availability.",
         "inputSchema":{"type":"object","properties":{},"additionalProperties":false}}
    ])
}

pub fn command_list(host: &CommandHost) -> Value {
    host.commands()
}

pub(crate) fn validate(name: &str, args: &Value) -> Result<(), String> {
    let object = args.as_object().ok_or("arguments must be an object")?;
    match name {
        "command_list" | "state_get" if !object.is_empty() => Err("tool takes no arguments".into()),
        "command_list" | "state_get" => Ok(()),
        "command_run" => {
            if object.keys().any(|key| key != "command" && key != "params") {
                return Err("unknown command_run argument".into());
            }
            if !args.get("command").is_some_and(Value::is_string) {
                return Err("command must be a string".into());
            }
            if args.get("params").is_some_and(|params| !params.is_object()) {
                return Err("command params must be an object".into());
            }
            Ok(())
        }
        _ => Err(format!("unknown tool: {name}")),
    }
}

pub(crate) fn call(host: &mut CommandHost, name: &str, args: &Value) -> Result<Value, String> {
    match name {
        "command_list" => Ok(command_list(host)),
        "state_get" => Ok(host.session().inspect()),
        "command_run" => {
            let command = args.get("command").and_then(Value::as_str).ok_or("command must be a string")?;
            let params = args.get("params").cloned().unwrap_or_else(|| json!({}));
            let result = host.execute(command, params).map_err(|error| error.to_string())?;
            serde_json::to_value(result).map_err(|error| error.to_string())
        }
        _ => Err(format!("unknown tool: {name}")),
    }
}
