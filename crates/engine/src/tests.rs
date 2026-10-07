use app_core::{AppError, MAX_HISTORY, MAX_INPUT_BYTES, MAX_NAME_BYTES};
use serde_json::{Value, json};

use crate::{Event, Session};

#[test]
fn registry_is_unique_and_metadata_serializes() {
    let session = Session::new();
    let mut ids = std::collections::HashSet::new();
    for command in session.commands() {
        assert!(ids.insert(command.id));
        assert_eq!(command.params["additionalProperties"], false);
        assert!(serde_json::to_value(command).is_ok());
    }
    assert_eq!(ids.len(), 8);
    assert!(!session.is_enabled("unknown"));
}

#[test]
fn undo_redo_branches_and_noop_preserves_history() {
    let mut session = Session::new();
    session.execute("demo.setName", json!({"name":"first"})).unwrap();
    session.execute("demo.incrementCounter", json!({"amount":5})).unwrap();
    session.execute("edit.undo", json!({})).unwrap();
    assert_eq!(session.model().counter, 0);
    assert_eq!(session.model().name, "first");
    session.execute("demo.incrementCounter", json!({"amount":0})).unwrap();
    assert!(session.is_enabled("edit.redo"));
    session.execute("edit.redo", json!({})).unwrap();
    assert_eq!(session.model().counter, 5);
    session.execute("edit.undo", json!({})).unwrap();
    session.execute("demo.setName", json!({"name":"branch"})).unwrap();
    assert!(!session.is_enabled("edit.redo"));
    assert!(matches!(session.execute("edit.redo", json!({})), Err(AppError::DisabledCommand(_))));
    session.execute("edit.undo", json!({})).unwrap();
    assert_eq!(session.model().name, "first");
}

#[test]
fn history_is_bounded_and_redo_restores_retained_states() {
    let mut session = Session::new();
    for _ in 0..MAX_HISTORY + 5 {
        session.execute("demo.incrementCounter", json!({})).unwrap();
    }
    for _ in 0..MAX_HISTORY {
        session.execute("edit.undo", json!({})).unwrap();
    }
    assert_eq!(session.model().counter, 5);
    assert!(!session.is_enabled("edit.undo"));
    for _ in 0..MAX_HISTORY {
        session.execute("edit.redo", json!({})).unwrap();
    }
    assert_eq!(session.model().counter, 105);
    assert!(!session.is_enabled("edit.redo"));
}

#[test]
fn save_open_and_new_reset_history() {
    let mut session = Session::new();
    session.execute("demo.setName", json!({"name":"saved"})).unwrap();
    let saved = session.execute("app.save", json!({})).unwrap();
    assert_eq!(saved.data, json!({"name":"saved", "counter":0}));
    assert_eq!(saved.events, vec![Event::Saved]);
    session.execute("demo.incrementCounter", json!({})).unwrap();
    session.execute("app.open", json!({"data":saved.data})).unwrap();
    assert_eq!(session.model().counter, 0);
    assert!(!session.is_enabled("edit.undo"));
    assert!(!session.is_enabled("edit.redo"));
    session.execute("app.new", json!({})).unwrap();
    assert_eq!(session.model(), &app_model::AppModel::default());
}

#[test]
fn hostile_params_never_panic_or_mutate() {
    let commands: Vec<_> = Session::new().commands().iter().map(|spec| spec.id).collect();
    let hostile = [
        Value::Null,
        json!(true),
        json!(5),
        json!([]),
        json!("text"),
        json!({"unexpected":1}),
        json!({"amount":1.5}),
        json!({"name":12}),
        json!({"data":{"name":"ok", "counter":0, "extra":true}}),
    ];
    for command in commands {
        for params in &hostile {
            let mut session = Session::new();
            let before = session.inspect();
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| session.execute(command, params.clone())));
            assert!(result.is_ok(), "command {command}");
            assert!(result.unwrap().is_err(), "command {command}: {params}");
            assert_eq!(session.inspect(), before);
        }
    }
}

#[test]
fn overflow_and_failed_open_preserve_state_and_history() {
    let mut session = Session::new();
    session.execute("app.open", json!({"data":{"name":"max", "counter":i64::MAX}})).unwrap();
    session.execute("demo.setName", json!({"name":"history"})).unwrap();
    let before = session.inspect();
    assert!(matches!(session.execute("demo.incrementCounter", json!({})), Err(AppError::LimitExceeded(_))));
    assert!(session.execute("app.open", json!({"data":{"name":"oops"}})).is_err());
    assert_eq!(session.inspect(), before);
    session.execute("edit.undo", json!({})).unwrap();
    assert_eq!(session.model().name, "max");
    session.execute("app.open", json!({"data":{"name":"min", "counter":i64::MIN}})).unwrap();
    assert!(matches!(session.execute("demo.incrementCounter", json!({"amount":-1})), Err(AppError::LimitExceeded(_))));
}

#[test]
fn name_limit_counts_utf8_bytes_and_accepts_exact_boundary() {
    let mut session = Session::new();
    let name = "é".repeat(MAX_NAME_BYTES / 2);
    session.execute("demo.setName", json!({"name":name})).unwrap();
    assert_eq!(session.model().name.len(), MAX_NAME_BYTES);
    let before = session.inspect();
    assert!(matches!(session.execute("demo.setName", json!({"name":format!("{name}x")})), Err(AppError::LimitExceeded(_))));
    assert_eq!(session.inspect(), before);
}

#[test]
fn input_limits_unknown_commands_and_view_state() {
    let mut session = Session::new();
    let before = session.inspect();
    assert!(matches!(session.execute("unknown", json!({})), Err(AppError::UnknownCommand(_))));
    assert!(matches!(session.execute("demo.setName", json!({"name":"x".repeat(MAX_INPUT_BYTES)})), Err(AppError::LimitExceeded(_))));
    let mut nested = Value::Null;
    for _ in 0..70 {
        nested = json!([nested]);
    }
    assert!(matches!(session.execute("app.new", nested), Err(AppError::LimitExceeded(_))));
    assert_eq!(session.inspect(), before);
    let result = session.execute("view.togglePanel", json!({})).unwrap();
    assert_eq!(result.events, vec![Event::ViewChanged]);
    assert_eq!(result.state["view"]["panel_visible"], false);
    assert!(!session.is_enabled("edit.undo"));
}

#[test]
fn model_sequences_are_rejected_without_resetting_history() {
    let mut session = Session::new();
    session.execute("demo.incrementCounter", json!({})).unwrap();
    let before = session.inspect();
    for data in [json!(["name", 0]), json!([]), Value::Null] {
        assert!(matches!(session.execute("app.open", json!({"data":data})), Err(AppError::InvalidParams(_))));
        assert_eq!(session.inspect(), before);
    }
    session.execute("edit.undo", json!({})).unwrap();
    assert_eq!(session.model().counter, 0);
}

#[test]
fn disabled_registry_entries_never_execute() {
    let mut session = Session::new();
    for command in ["edit.undo", "edit.redo"] {
        assert!(!session.is_enabled(command));
        // Availability is checked before handlers parse their command-specific parameters.
        assert!(matches!(session.execute(command, json!({"unexpected":true})), Err(AppError::DisabledCommand(_))));
    }
    assert_eq!(session.model(), &app_model::AppModel::default());
}
