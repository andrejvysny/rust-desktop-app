//! JSON control dispatch shared by native automation and headless tests.
use app_platform::CommandHost;
use serde_json::{Value, json};

pub use app_core::MAX_INPUT_BYTES;

pub fn handle(host: &mut CommandHost, request: &Value) -> Value {
    let id = request.get("id").cloned().unwrap_or(Value::Null);
    let method = request.get("method").and_then(Value::as_str);
    let result = match method {
        Some("engine.commands") => Ok(host.commands()),
        Some("state.get") => Ok(host.session().inspect()),
        Some("engine.execute") => execute(host, request),
        _ => Err("unknown control method".to_owned()),
    };
    match result {
        Ok(result) => json!({"id":id,"result":result}),
        Err(message) => json!({"id":id,"error":{"code":"control_error","message":message}}),
    }
}

fn execute(host: &mut CommandHost, request: &Value) -> Result<Value, String> {
    let params = request.get("params").ok_or("missing params")?;
    let command = params.get("command").and_then(Value::as_str).ok_or("missing command")?;
    let args = params.get("params").cloned().unwrap_or_else(|| json!({}));
    let result = host.execute(command, args).map_err(|error| error.to_string())?;
    serde_json::to_value(result).map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn control_changes_the_same_session() {
        let mut host = CommandHost::default();
        let response = handle(&mut host, &json!({"id":1,"method":"engine.execute","params":{"command":"demo.incrementCounter","params":{}}}));
        assert!(response.get("result").is_some());
        assert_eq!(host.session().model().counter, 1);
        assert!(handle(&mut host, &json!({"method":"missing"})).get("error").is_some());
    }
}
