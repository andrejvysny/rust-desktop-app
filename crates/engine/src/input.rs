use std::io::{self, Write};

use app_core::{AppError, MAX_INPUT_BYTES};
use serde::de::DeserializeOwned;
use serde_json::Value;

pub(crate) fn parse<T: DeserializeOwned>(params: Value) -> Result<T, AppError> {
    if !params.is_object() {
        return Err(AppError::InvalidParams("parameters must be a JSON object".to_owned()));
    }
    serde_json::from_value(params).map_err(|error| AppError::InvalidParams(error.to_string()))
}

pub(crate) fn check_size(params: &Value) -> Result<(), AppError> {
    check_depth(params)?;
    // Count encoded bytes without allocating another copy of untrusted input.
    let mut counter = ByteCounter(0);
    serde_json::to_writer(&mut counter, params).map_err(|_| AppError::LimitExceeded(format!("input exceeds {MAX_INPUT_BYTES} bytes")))
}

fn check_depth(params: &Value) -> Result<(), AppError> {
    let mut pending = vec![(params, 0usize)];
    let mut nodes = 0usize;
    while let Some((value, depth)) = pending.pop() {
        nodes = nodes.saturating_add(1);
        if depth > 64 || nodes > MAX_INPUT_BYTES {
            return Err(AppError::LimitExceeded("input nesting or node limit exceeded".to_owned()));
        }
        let children = match value {
            Value::Array(values) => values.len(),
            Value::Object(values) => values.len(),
            _ => 0,
        };
        if pending.len().saturating_add(children).saturating_add(nodes) > MAX_INPUT_BYTES {
            return Err(AppError::LimitExceeded("input node limit exceeded".to_owned()));
        }
        match value {
            Value::Array(values) => pending.extend(values.iter().map(|value| (value, depth + 1))),
            Value::Object(values) => pending.extend(values.values().map(|value| (value, depth + 1))),
            _ => {}
        }
    }
    Ok(())
}

struct ByteCounter(usize);

impl Write for ByteCounter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let next = self.0.saturating_add(bytes.len());
        if next > MAX_INPUT_BYTES {
            return Err(io::Error::other("input size limit exceeded"));
        }
        self.0 = next;
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
