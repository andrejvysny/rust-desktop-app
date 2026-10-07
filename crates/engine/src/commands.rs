use app_core::AppError;
use serde::Serialize;
use serde_json::{Value, json};

use crate::Session;

pub(crate) type Handler = fn(&mut Session, Value) -> Result<(Vec<crate::Event>, Value), AppError>;

#[derive(Debug, Clone, Serialize)]
pub struct CommandSpec {
    pub id: &'static str,
    pub label: &'static str,
    pub description: &'static str,
    pub params: Value,
    pub undoable: bool,
    #[serde(skip)]
    pub(crate) run: Handler,
    #[serde(skip)]
    pub(crate) enabled: fn(&Session) -> bool,
}

/// Extend the compile-time command list with metadata and a Session handler.
/// Each entry owns the execution and availability functions used by every frontend.
#[derive(Debug, Clone)]
pub struct CommandRegistry {
    specs: Vec<CommandSpec>,
}

impl Default for CommandRegistry {
    fn default() -> Self {
        let mut specs = file_history_commands();
        specs.extend(demo_view_commands());
        Self { specs }
    }
}

fn empty_schema() -> Value {
    json!({"type":"object", "properties":{}, "additionalProperties":false})
}

fn name_schema() -> Value {
    json!({"type":"string", "maxLength":256,
        "description":"Maximum 256 UTF-8 bytes; this may be fewer than 256 Unicode characters."})
}

fn file_history_commands() -> Vec<CommandSpec> {
    let empty = empty_schema();
    let model = json!({"type":"object", "required":["name","counter"],
        "properties":{"name":name_schema(),
            "counter":{"type":"integer", "minimum":i64::MIN, "maximum":i64::MAX}},
        "additionalProperties":false});
    vec![
        spec("app.new", "New", "Reset state and history.", empty.clone(), false, Session::run_new),
        spec(
            "app.open",
            "Open",
            "Load validated state and reset history.",
            json!({"type":"object", "properties":{"data":model},
                "required":["data"], "additionalProperties":false}),
            false,
            Session::open,
        ),
        spec("app.save", "Save", "Return state for frontend storage.", empty.clone(), false, Session::run_save),
        spec("edit.undo", "Undo", "Restore the previous state.", empty.clone(), false, Session::run_undo).with_enabled(Session::can_undo),
        spec("edit.redo", "Redo", "Restore the next state.", empty, false, Session::run_redo).with_enabled(Session::can_redo),
    ]
}

fn demo_view_commands() -> Vec<CommandSpec> {
    vec![
        spec(
            "demo.incrementCounter",
            "Change counter",
            "Add an amount; default is one.",
            json!({"type":"object", "properties":{"amount":{"type":"integer",
                "default":1, "minimum":i64::MIN, "maximum":i64::MAX}},
                "additionalProperties":false}),
            true,
            Session::increment,
        ),
        spec(
            "demo.setName",
            "Set name",
            "Set the demonstration name (256 UTF-8 bytes maximum).",
            json!({"type":"object", "properties":{"name":name_schema()},
                "required":["name"], "additionalProperties":false}),
            true,
            Session::set_name,
        ),
        spec("view.togglePanel", "Toggle panel", "Toggle session presentation state.", empty_schema(), false, Session::toggle_panel),
    ]
}

impl CommandRegistry {
    pub fn commands(&self) -> &[CommandSpec] {
        &self.specs
    }

    pub fn get(&self, id: &str) -> Option<&CommandSpec> {
        self.specs.iter().find(|spec| spec.id == id)
    }
}

fn spec(id: &'static str, label: &'static str, description: &'static str, params: Value, undoable: bool, run: Handler) -> CommandSpec {
    CommandSpec { id, label, description, params, undoable, run, enabled: |_| true }
}

impl CommandSpec {
    fn with_enabled(mut self, enabled: fn(&Session) -> bool) -> Self {
        self.enabled = enabled;
        self
    }
}
