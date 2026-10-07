//! Reusable acceptance helpers; consumers use this crate only as a dev-dependency.

use app_core::AppError;
use app_platform::CommandHost;
use serde_json::{Value, json};

pub fn run_commands(commands: &[(&str, Value)]) -> Result<CommandHost, AppError> {
    let mut host = CommandHost::default();
    for (id, params) in commands {
        host.execute(id, params.clone())?;
    }
    Ok(host)
}

/// Representative malformed input shared by all command transport tests.
pub fn hostile_params() -> Vec<Value> {
    vec![Value::Null, json!(false), json!(7), json!([]), json!({"unexpected": true}), json!({"name": 7}), json!({"amount": "bad"})]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_commands_work_without_a_window() {
        let host = run_commands(&[("demo.incrementCounter", json!({})), ("demo.setName", json!({"name": "Test"}))]).unwrap();
        assert_eq!(host.session().model().counter, 1);
        assert_eq!(host.session().model().name, "Test");
    }
}
