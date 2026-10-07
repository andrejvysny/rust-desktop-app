use std::collections::VecDeque;

use app_core::{AppError, MAX_HISTORY};
use app_model::AppModel;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::{CommandRegistry, CommandSpec, input};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Event {
    StateChanged,
    Opened,
    Saved,
    HistoryChanged,
    ViewChanged,
}

#[derive(Debug, Clone, Serialize)]
pub struct CommandResult {
    pub state: Value,
    pub events: Vec<Event>,
    pub data: Value,
}

#[derive(Debug, Clone)]
pub struct Session {
    model: AppModel,
    registry: CommandRegistry,
    undo: VecDeque<AppModel>,
    redo: VecDeque<AppModel>,
    panel_visible: bool,
}

impl Default for Session {
    fn default() -> Self {
        Self { model: AppModel::default(), registry: CommandRegistry::default(), undo: VecDeque::new(), redo: VecDeque::new(), panel_visible: true }
    }
}

impl Session {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn model(&self) -> &AppModel {
        &self.model
    }

    pub fn commands(&self) -> &[CommandSpec] {
        self.registry.commands()
    }

    pub fn inspect(&self) -> Value {
        json!({"model":self.model, "can_undo":!self.undo.is_empty(),
            "can_redo":!self.redo.is_empty(), "view":{"panel_visible":self.panel_visible}})
    }

    pub fn is_enabled(&self, command: &str) -> bool {
        self.registry.get(command).is_some_and(|spec| (spec.enabled)(self))
    }

    pub fn execute(&mut self, command: &str, params: Value) -> Result<CommandResult, AppError> {
        let spec = self.registry.get(command).ok_or_else(|| AppError::UnknownCommand(command.to_owned()))?;
        let run = spec.run;
        input::check_size(&params)?;
        if !(spec.enabled)(self) {
            return Err(AppError::DisabledCommand(command.to_owned()));
        }
        let (events, data) = run(self, params)?;
        Ok(CommandResult { state: self.inspect(), events, data })
    }

    pub(crate) fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }
    pub(crate) fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    pub(crate) fn run_new(&mut self, params: Value) -> Result<(Vec<Event>, Value), AppError> {
        let _: EmptyParams = input::parse(params)?;
        self.reset(AppModel::default());
        Ok((vec![Event::StateChanged, Event::HistoryChanged], Value::Null))
    }

    pub(crate) fn run_save(&mut self, params: Value) -> Result<(Vec<Event>, Value), AppError> {
        let _: EmptyParams = input::parse(params)?;
        Ok((vec![Event::Saved], json!(self.model)))
    }

    pub(crate) fn run_undo(&mut self, params: Value) -> Result<(Vec<Event>, Value), AppError> {
        let _: EmptyParams = input::parse(params)?;
        self.undo()?;
        Ok((vec![Event::StateChanged, Event::HistoryChanged], Value::Null))
    }

    pub(crate) fn run_redo(&mut self, params: Value) -> Result<(Vec<Event>, Value), AppError> {
        let _: EmptyParams = input::parse(params)?;
        self.redo()?;
        Ok((vec![Event::StateChanged, Event::HistoryChanged], Value::Null))
    }

    pub(crate) fn toggle_panel(&mut self, params: Value) -> Result<(Vec<Event>, Value), AppError> {
        let _: EmptyParams = input::parse(params)?;
        self.panel_visible = !self.panel_visible;
        Ok((vec![Event::ViewChanged], Value::Null))
    }

    pub(crate) fn open(&mut self, params: Value) -> Result<(Vec<Event>, Value), AppError> {
        let params: OpenParams = input::parse(params)?;
        let model: AppModel = input::parse(params.data)?;
        model.validate()?;
        self.reset(model);
        Ok((vec![Event::Opened, Event::StateChanged, Event::HistoryChanged], Value::Null))
    }

    pub(crate) fn increment(&mut self, params: Value) -> Result<(Vec<Event>, Value), AppError> {
        let params: IncrementParams = input::parse(params)?;
        let counter = self.model.counter.checked_add(params.amount).ok_or_else(|| AppError::LimitExceeded("counter overflow".to_owned()))?;
        let next = AppModel { counter, ..self.model.clone() };
        Ok((self.update(next), Value::Null))
    }

    pub(crate) fn set_name(&mut self, params: Value) -> Result<(Vec<Event>, Value), AppError> {
        let params: NameParams = input::parse(params)?;
        let next = AppModel { name: params.name, ..self.model.clone() };
        next.validate()?;
        Ok((self.update(next), Value::Null))
    }

    fn update(&mut self, next: AppModel) -> Vec<Event> {
        if next == self.model {
            return Vec::new();
        }
        push_history(&mut self.undo, self.model.clone());
        self.model = next;
        self.redo.clear();
        vec![Event::StateChanged, Event::HistoryChanged]
    }

    fn reset(&mut self, model: AppModel) {
        self.model = model;
        self.undo.clear();
        self.redo.clear();
    }

    fn undo(&mut self) -> Result<(), AppError> {
        let previous = self.undo.pop_back().ok_or_else(|| AppError::DisabledCommand("edit.undo".to_owned()))?;
        push_history(&mut self.redo, std::mem::replace(&mut self.model, previous));
        Ok(())
    }

    fn redo(&mut self) -> Result<(), AppError> {
        let next = self.redo.pop_back().ok_or_else(|| AppError::DisabledCommand("edit.redo".to_owned()))?;
        push_history(&mut self.undo, std::mem::replace(&mut self.model, next));
        Ok(())
    }
}

fn push_history(history: &mut VecDeque<AppModel>, model: AppModel) {
    if history.len() >= MAX_HISTORY {
        history.pop_front();
    }
    history.push_back(model);
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EmptyParams {}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OpenParams {
    data: Value,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct IncrementParams {
    #[serde(default = "default_amount")]
    amount: i64,
}

fn default_amount() -> i64 {
    1
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NameParams {
    name: String,
}
