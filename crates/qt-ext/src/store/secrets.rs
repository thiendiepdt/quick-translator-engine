//! Chỗ giữ giá trị bí mật của extension (trường `secret` của auth và cài đặt). Windows: Credential Manager qua
//! `keyring`, mỗi giá trị một mục. Máy khác: file JSON trong thư mục dữ liệu của extension (form hiện cảnh báo).
//! Test: bộ nhớ.

use super::{io_error, write_atomic};
use crate::ExtError;
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub enum SecretBackend {
    Keyring,
    File(PathBuf),
    Memory(Arc<Mutex<HashMap<String, String>>>),
}

/// Ghi file khoá theo kiểu đọc → sửa → ghi; khoá chung để hai lượt trong app không đè nhau.
static FILE_LOCK: Mutex<()> = Mutex::new(());

fn read_file(path: &PathBuf) -> Map<String, Value> {
    std::fs::read_to_string(path).ok().and_then(|text| serde_json::from_str(&text).ok()).unwrap_or_default()
}

impl SecretBackend {
    /// Windows: Credential Manager; máy khác: `file`.
    pub fn platform(file: PathBuf) -> Self {
        if cfg!(windows) {
            SecretBackend::Keyring
        } else {
            SecretBackend::File(file)
        }
    }

    pub fn memory() -> Self {
        SecretBackend::Memory(Arc::default())
    }

    /// "keyring" | "file" | "memory" — form hiện cảnh báo khi khác "keyring".
    pub fn kind(&self) -> &'static str {
        match self {
            SecretBackend::Keyring => "keyring",
            SecretBackend::File(_) => "file",
            SecretBackend::Memory(_) => "memory",
        }
    }

    pub fn get(&self, name: &str) -> Result<Option<String>, ExtError> {
        match self {
            SecretBackend::Keyring => native::get(name),
            SecretBackend::File(path) => {
                let _guard = FILE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
                Ok(read_file(path).get(name).and_then(Value::as_str).map(str::to_string))
            }
            SecretBackend::Memory(map) => Ok(map.lock().unwrap_or_else(|e| e.into_inner()).get(name).cloned()),
        }
    }

    pub fn set(&self, name: &str, value: &str) -> Result<(), ExtError> {
        match self {
            SecretBackend::Keyring => native::set(name, value),
            SecretBackend::File(path) => {
                let _guard = FILE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
                let mut map = read_file(path);
                map.insert(name.to_string(), Value::String(value.to_string()));
                write_atomic(path, serde_json::to_string_pretty(&map).map_err(|e| ExtError::Store(e.to_string()))?.as_bytes())
            }
            SecretBackend::Memory(map) => {
                map.lock().unwrap_or_else(|e| e.into_inner()).insert(name.to_string(), value.to_string());
                Ok(())
            }
        }
    }

    /// Không có thì thôi, không báo lỗi.
    pub fn delete(&self, name: &str) -> Result<(), ExtError> {
        match self {
            SecretBackend::Keyring => native::delete(name),
            SecretBackend::File(path) => {
                let _guard = FILE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
                let mut map = read_file(path);
                if map.remove(name).is_none() {
                    return Ok(());
                }
                if map.is_empty() {
                    return match std::fs::remove_file(path) {
                        Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(io_error(path)(e)),
                        _ => Ok(()),
                    };
                }
                write_atomic(path, serde_json::to_string_pretty(&map).map_err(|e| ExtError::Store(e.to_string()))?.as_bytes())
            }
            SecretBackend::Memory(map) => {
                map.lock().unwrap_or_else(|e| e.into_inner()).remove(name);
                Ok(())
            }
        }
    }
}

#[cfg(windows)]
mod native {
    use crate::ExtError;
    use keyring::{Entry, Error};

    const SERVICE: &str = "qt-ext";

    fn store_error(error: Error) -> ExtError {
        match error {
            Error::TooLong(..) => ExtError::Store("giá trị quá dài cho Credential Manager (tối đa 1280 ký tự)".into()),
            other => ExtError::Store(format!("Credential Manager: {other}")),
        }
    }

    fn entry(name: &str) -> Result<Entry, ExtError> {
        Entry::new_with_target(name, SERVICE, name).map_err(store_error)
    }

    pub fn get(name: &str) -> Result<Option<String>, ExtError> {
        match entry(name)?.get_password() {
            Ok(value) => Ok(Some(value)),
            Err(Error::NoEntry) => Ok(None),
            Err(e) => Err(store_error(e)),
        }
    }

    pub fn set(name: &str, value: &str) -> Result<(), ExtError> {
        entry(name)?.set_password(value).map_err(store_error)
    }

    pub fn delete(name: &str) -> Result<(), ExtError> {
        match entry(name)?.delete_credential() {
            Ok(()) | Err(Error::NoEntry) => Ok(()),
            Err(e) => Err(store_error(e)),
        }
    }
}

#[cfg(not(windows))]
mod native {
    use crate::ExtError;

    fn missing() -> ExtError {
        ExtError::Store("máy này không có kho khoá".into())
    }

    pub fn get(_: &str) -> Result<Option<String>, ExtError> {
        Err(missing())
    }

    pub fn set(_: &str, _: &str) -> Result<(), ExtError> {
        Err(missing())
    }

    pub fn delete(_: &str) -> Result<(), ExtError> {
        Err(missing())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn roundtrip(backend: &SecretBackend) {
        assert_eq!(backend.get("qt-ext/a/auth/k/apiKey").unwrap(), None);
        backend.set("qt-ext/a/auth/k/apiKey", "khoá-1").unwrap();
        backend.set("qt-ext/a/setting/token", "t").unwrap();
        assert_eq!(backend.get("qt-ext/a/auth/k/apiKey").unwrap().as_deref(), Some("khoá-1"));
        backend.set("qt-ext/a/auth/k/apiKey", "khoá-2").unwrap();
        assert_eq!(backend.get("qt-ext/a/auth/k/apiKey").unwrap().as_deref(), Some("khoá-2"));
        backend.delete("qt-ext/a/auth/k/apiKey").unwrap();
        backend.delete("qt-ext/a/auth/k/apiKey").unwrap();
        assert_eq!(backend.get("qt-ext/a/auth/k/apiKey").unwrap(), None);
        assert_eq!(backend.get("qt-ext/a/setting/token").unwrap().as_deref(), Some("t"));
    }

    #[test]
    fn file_va_memory_ghi_doc_xoa() {
        roundtrip(&SecretBackend::memory());
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("x").join("secrets.json");
        roundtrip(&SecretBackend::File(path.clone()));
        SecretBackend::File(path.clone()).delete("qt-ext/a/setting/token").unwrap();
        assert!(!path.exists(), "hết khoá thì xoá file");
    }

    /// Ghi thật vào Credential Manager rồi xoá. Chạy tay: `cargo test -p qt-ext keyring_that -- --ignored`.
    #[cfg(windows)]
    #[test]
    #[ignore]
    fn keyring_that() {
        let name = format!("qt-ext/io.github.test.keyring-{}/auth/k/apiKey", std::process::id());
        let backend = SecretBackend::Keyring;
        backend.delete(&name).unwrap();
        assert_eq!(backend.get(&name).unwrap(), None);
        backend.set(&name, "giá trị thử").unwrap();
        assert_eq!(backend.get(&name).unwrap().as_deref(), Some("giá trị thử"));
        backend.delete(&name).unwrap();
        assert_eq!(backend.get(&name).unwrap(), None);
        assert!(backend.set(&name, &"x".repeat(1300)).unwrap_err().to_string().contains("1280"));
    }
}
