//! Frontend services. The engine never imports this crate.

mod host;
mod memory;
#[cfg(not(target_arch = "wasm32"))]
mod native;

pub use host::CommandHost;
pub use memory::MemoryStorage;
#[cfg(not(target_arch = "wasm32"))]
pub use native::NativeStorage;

use app_core::{AppError, MAX_INPUT_BYTES};

/// Persistence implementations may use files, memory, or a browser service.
pub trait Storage {
    fn read(&mut self, path: &str) -> Result<Vec<u8>, AppError>;
    fn write(&mut self, path: &str, data: &[u8]) -> Result<(), AppError>;
}

/// Interactive frontends select paths before dispatching a command.
/// Headless callers supply paths directly and never invoke this service.
pub trait FileDialogs {
    fn open(&mut self) -> Result<Option<String>, AppError>;
    fn save(&mut self) -> Result<Option<String>, AppError>;
}

pub fn validate_path(path: &str) -> Result<(), AppError> {
    if path.is_empty() || path.len() > 4096 || path.contains('\0') {
        return Err(AppError::InvalidParams("path must contain 1–4096 bytes and no NUL".into()));
    }
    Ok(())
}

fn validate_data(data: &[u8]) -> Result<(), AppError> {
    if data.len() > MAX_INPUT_BYTES {
        return Err(AppError::LimitExceeded(format!("storage input exceeds {MAX_INPUT_BYTES} bytes")));
    }
    Ok(())
}
