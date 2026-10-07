use std::collections::BTreeMap;

use app_core::{AppError, MAX_INPUT_BYTES};

use crate::{Storage, validate_data, validate_path};

/// Bounded temporary persistence for tests and the browser demo.
#[derive(Default)]
pub struct MemoryStorage {
    files: BTreeMap<String, Vec<u8>>,
}

impl Storage for MemoryStorage {
    fn read(&mut self, path: &str) -> Result<Vec<u8>, AppError> {
        validate_path(path)?;
        self.files.get(path).cloned().ok_or_else(|| AppError::Storage(format!("file not found: {path}")))
    }

    fn write(&mut self, path: &str, data: &[u8]) -> Result<(), AppError> {
        validate_path(path)?;
        validate_data(data)?;
        if !self.files.contains_key(path) && self.files.len() >= 64 {
            return Err(AppError::LimitExceeded("memory storage allows at most 64 files".into()));
        }
        let other_bytes: usize = self.files.iter().filter(|(key, _)| key.as_str() != path).map(|(_, value)| value.len()).sum();
        if other_bytes.saturating_add(data.len()) > MAX_INPUT_BYTES * 8 {
            return Err(AppError::LimitExceeded("memory storage exceeds 8 MiB".into()));
        }
        self.files.insert(path.to_owned(), data.to_vec());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_limits_reject_without_replacing_existing_files() {
        let mut storage = MemoryStorage::default();
        storage.write("x", b"old").unwrap();
        assert!(storage.write("x", &vec![0; MAX_INPUT_BYTES + 1]).is_err());
        assert_eq!(storage.read("x").unwrap(), b"old");
        assert!(storage.write("", b"new").is_err());
        for i in 1..64 {
            storage.write(&i.to_string(), b"v").unwrap();
        }
        assert!(storage.write("overflow", b"v").is_err());
    }
}
