//! Minimal serializable state. Replace this model with application data.
#![forbid(unsafe_code)]

use app_core::{AppError, MAX_NAME_BYTES};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AppModel {
    pub name: String,
    pub counter: i64,
}

impl AppModel {
    pub fn validate(&self) -> Result<(), AppError> {
        if self.name.len() > MAX_NAME_BYTES {
            return Err(AppError::LimitExceeded(format!("name exceeds {MAX_NAME_BYTES} UTF-8 bytes")));
        }
        Ok(())
    }
}
