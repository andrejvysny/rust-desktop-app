use app_core::AppError;
use app_engine::{CommandResult, Session};
use serde_json::{Value, json};

use crate::{MemoryStorage, Storage, validate_data, validate_path};

/// Shared boundary for file-oriented commands in every frontend.
pub struct CommandHost {
    session: Session,
    storage: Box<dyn Storage>,
}

impl Default for CommandHost {
    fn default() -> Self {
        Self::new(Box::<MemoryStorage>::default())
    }
}

impl CommandHost {
    pub fn new(storage: Box<dyn Storage>) -> Self {
        Self { session: Session::new(), storage }
    }

    pub fn session(&self) -> &Session {
        &self.session
    }

    /// Discovery includes the storage adapter's path mode and live enablement.
    pub fn commands(&self) -> Value {
        Value::Array(
            self.session
                .commands()
                .iter()
                .map(|spec| {
                    let params = if matches!(spec.id, "app.open" | "app.save") {
                        json!({"oneOf": [spec.params, {
                            "type": "object", "properties": {"path": {"type": "string", "minLength": 1, "maxLength": 4096}},
                            "required": ["path"], "additionalProperties": false
                        }]})
                    } else {
                        spec.params.clone()
                    };
                    json!({"id": spec.id, "label": spec.label, "description": spec.description,
                "params": params, "undoable": spec.undoable, "enabled": self.session.is_enabled(spec.id)})
                })
                .collect(),
        )
    }

    pub fn execute(&mut self, command: &str, params: Value) -> Result<CommandResult, AppError> {
        if matches!(command, "app.open" | "app.save") && params.get("path").is_some() {
            let path = storage_path(&params)?;
            return self.execute_path(command, path);
        }
        self.session.execute(command, params)
    }

    fn execute_path(&mut self, command: &str, path: &str) -> Result<CommandResult, AppError> {
        validate_path(path)?;
        if command == "app.open" {
            let bytes = self.storage.read(path)?;
            validate_data(&bytes)?;
            let data: Value = serde_json::from_slice(&bytes).map_err(|error| AppError::Serialization(error.to_string()))?;
            return self.session.execute(command, json!({"data": data}));
        }
        let result = self.session.execute(command, json!({}))?;
        let bytes = serde_json::to_vec_pretty(&result.data).map_err(|error| AppError::Serialization(error.to_string()))?;
        validate_data(&bytes)?;
        self.storage.write(path, &bytes)?;
        Ok(result)
    }
}

fn storage_path(params: &Value) -> Result<&str, AppError> {
    let object = params.as_object().ok_or_else(|| AppError::InvalidParams("parameters must be an object".into()))?;
    if object.len() != 1 {
        return Err(AppError::InvalidParams("path mode accepts only `path`".into()));
    }
    params.get("path").and_then(Value::as_str).ok_or_else(|| AppError::InvalidParams("path must be a string".into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn storage_roundtrip_uses_engine_commands() {
        let mut host = CommandHost::default();
        host.execute("demo.setName", json!({"name": "Sample"})).unwrap();
        host.execute("demo.incrementCounter", json!({"amount": 5})).unwrap();
        host.execute("app.save", json!({"path": "sample.json"})).unwrap();
        host.execute("app.new", json!({})).unwrap();
        host.execute("app.open", json!({"path": "sample.json"})).unwrap();
        assert_eq!(host.session().model().name, "Sample");
        assert_eq!(host.session().model().counter, 5);
        assert!(!host.session().is_enabled("edit.undo"));
    }

    #[test]
    fn bad_open_and_ambiguous_paths_preserve_state() {
        let mut storage = MemoryStorage::default();
        storage.write("bad.json", b"not json").unwrap();
        let mut host = CommandHost::new(Box::new(storage));
        host.execute("demo.incrementCounter", json!({})).unwrap();
        for params in [json!({"path": "bad.json"}), json!({"path": 7}), json!({"path": "x", "data": {}})] {
            assert!(host.execute("app.open", params).is_err());
            assert_eq!(host.session().model().counter, 1);
            assert!(host.session().is_enabled("edit.undo"));
        }
    }

    struct FailedStorage;
    impl Storage for FailedStorage {
        fn read(&mut self, _: &str) -> Result<Vec<u8>, AppError> {
            Err(AppError::Storage("read failed".into()))
        }
        fn write(&mut self, _: &str, _: &[u8]) -> Result<(), AppError> {
            Err(AppError::Storage("write failed".into()))
        }
    }

    #[test]
    fn save_failure_preserves_history() {
        let mut host = CommandHost::new(Box::new(FailedStorage));
        host.execute("demo.incrementCounter", json!({})).unwrap();
        let before = host.session().inspect();
        assert!(host.execute("app.save", json!({"path": "x"})).is_err());
        assert_eq!(host.session().inspect(), before);
    }

    #[test]
    fn discovery_exposes_file_mode_and_live_enablement() {
        let mut host = CommandHost::default();
        let commands = host.commands();
        let save = commands.as_array().unwrap().iter().find(|spec| spec["id"] == "app.save").unwrap();
        assert_eq!(save["params"]["oneOf"][1]["required"], json!(["path"]));
        host.execute("demo.incrementCounter", json!({})).unwrap();
        let commands = host.commands();
        let undo = commands.as_array().unwrap().iter().find(|spec| spec["id"] == "edit.undo").unwrap();
        assert_eq!(undo["enabled"], true);
    }
}
