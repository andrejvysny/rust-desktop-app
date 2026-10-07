use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use app_core::{AppError, MAX_INPUT_BYTES};

use crate::{Storage, validate_data, validate_path};

static NEXT_TEMP: AtomicU64 = AtomicU64::new(0);

/// Native persistence. Saves replace a file only after its new content is flushed.
pub struct NativeStorage;

impl Storage for NativeStorage {
    fn read(&mut self, path: &str) -> Result<Vec<u8>, AppError> {
        validate_path(path)?;
        let file = File::open(path).map_err(storage_error)?;
        let mut bytes = Vec::new();
        file.take(MAX_INPUT_BYTES as u64 + 1).read_to_end(&mut bytes).map_err(storage_error)?;
        validate_data(&bytes)?;
        Ok(bytes)
    }

    fn write(&mut self, path: &str, data: &[u8]) -> Result<(), AppError> {
        validate_path(path)?;
        validate_data(data)?;
        let destination = Path::new(path);
        let permissions = match std::fs::metadata(destination) {
            Ok(metadata) => Some(metadata.permissions()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
            Err(error) => return Err(storage_error(error)),
        };
        let (temporary, mut file) = temporary_file(destination)?;
        let result = file.write_all(data).and_then(|()| {
            if let Some(permissions) = permissions {
                file.set_permissions(permissions)?;
            }
            file.sync_all()
        });
        drop(file);
        let result = result.and_then(|()| std::fs::rename(&temporary, destination));
        if result.is_err() {
            let _ = std::fs::remove_file(&temporary);
        }
        result.map_err(storage_error)
    }
}

fn temporary_file(destination: &Path) -> Result<(PathBuf, File), AppError> {
    let parent = destination.parent().filter(|path| !path.as_os_str().is_empty()).unwrap_or_else(|| Path::new("."));
    for _ in 0..8 {
        let id = NEXT_TEMP.fetch_add(1, Ordering::Relaxed);
        let path = parent.join(format!(".app-save-{}-{id}.tmp", std::process::id()));
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        match options.open(&path) {
            Ok(file) => return Ok((path, file)),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(storage_error(error)),
        }
    }
    Err(AppError::Storage("could not reserve a temporary save file".into()))
}

fn storage_error(error: std::io::Error) -> AppError {
    AppError::Storage(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_save_overwrite_and_oversized_read() {
        let id = NEXT_TEMP.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!("app-storage-test-{}-{id}.json", std::process::id()));
        let path_string = path.to_str().unwrap();
        let mut storage = NativeStorage;
        storage.write(path_string, b"first").unwrap();
        storage.write(path_string, b"second").unwrap();
        assert_eq!(storage.read(path_string).unwrap(), b"second");
        std::fs::write(&path, vec![0; MAX_INPUT_BYTES + 1]).unwrap();
        assert!(matches!(storage.read(path_string), Err(AppError::LimitExceeded(_))));
        std::fs::remove_file(path).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn saves_keep_private_files_private() {
        use std::os::unix::fs::PermissionsExt;
        let id = NEXT_TEMP.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!("app-private-test-{}-{id}.json", std::process::id()));
        let path_string = path.to_str().unwrap();
        let mut storage = NativeStorage;
        storage.write(path_string, b"first").unwrap();
        assert_eq!(std::fs::metadata(&path).unwrap().permissions().mode() & 0o777, 0o600);
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o640)).unwrap();
        storage.write(path_string, b"second").unwrap();
        assert_eq!(std::fs::metadata(&path).unwrap().permissions().mode() & 0o777, 0o640);
        std::fs::remove_file(path).unwrap();
    }
}
