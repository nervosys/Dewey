//! State persistence — save and restore application state.
//!
//! Provides helpers to serialize model state to disk and reload it on startup.
//! Works with any type implementing `serde::Serialize` + `serde::Deserialize`.

use std::path::{Path, PathBuf};

/// Errors from persistence operations.
#[derive(Debug)]
pub enum PersistError {
    Io(std::io::Error),
    Json(serde_json::Error),
}

impl std::fmt::Display for PersistError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PersistError::Io(e) => write!(f, "Persistence I/O error: {e}"),
            PersistError::Json(e) => write!(f, "Persistence JSON error: {e}"),
        }
    }
}

impl std::error::Error for PersistError {}

impl From<std::io::Error> for PersistError {
    fn from(e: std::io::Error) -> Self {
        PersistError::Io(e)
    }
}

impl From<serde_json::Error> for PersistError {
    fn from(e: serde_json::Error) -> Self {
        PersistError::Json(e)
    }
}

/// A state store that persists JSON-serializable state to a file.
///
/// # Example
/// ```rust,no_run
/// use dewey::util::persistence::StateStore;
/// use serde::{Serialize, Deserialize};
///
/// #[derive(Serialize, Deserialize)]
/// struct AppState { count: i32, name: String }
///
/// let store = StateStore::new("my-app");
/// let state = AppState { count: 42, name: "hello".into() };
/// store.save(&state).unwrap();
///
/// let loaded: AppState = store.load().unwrap();
/// assert_eq!(loaded.count, 42);
/// ```
pub struct StateStore {
    path: PathBuf,
}

impl StateStore {
    /// Create a new state store. The file is placed in the OS-appropriate
    /// data directory (e.g., `~/.local/share/<app_name>/state.json` on Linux,
    /// `%APPDATA%/<app_name>/state.json` on Windows).
    ///
    /// Falls back to `./<app_name>_state.json` if the data dir can't be determined.
    pub fn new(app_name: &str) -> Self {
        let dir = std::env::var("APPDATA")
            .or_else(|_| std::env::var("XDG_DATA_HOME"))
            .or_else(|_| {
                std::env::var("HOME").map(|h| {
                    let mut p = PathBuf::from(h);
                    p.push(".local/share");
                    p.to_string_lossy().into_owned()
                })
            })
            .map(PathBuf::from)
            .unwrap_or_default();

        let app_dir = if dir.as_os_str().is_empty() {
            PathBuf::from(".")
        } else {
            dir.join(app_name)
        };

        Self {
            path: app_dir.join("state.json"),
        }
    }

    /// Create a state store at a specific file path.
    pub fn with_path(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    /// The file path where state is stored.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Save state to disk as JSON.
    pub fn save<T: serde::Serialize>(&self, state: &T) -> Result<(), PersistError> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(state)?;
        std::fs::write(&self.path, json)?;
        Ok(())
    }

    /// Load state from disk.
    pub fn load<T: serde::de::DeserializeOwned>(&self) -> Result<T, PersistError> {
        let data = std::fs::read_to_string(&self.path)?;
        let state = serde_json::from_str(&data)?;
        Ok(state)
    }

    /// Load state from disk, returning `None` if the file doesn't exist.
    pub fn load_or_none<T: serde::de::DeserializeOwned>(&self) -> Result<Option<T>, PersistError> {
        match std::fs::read_to_string(&self.path) {
            Ok(data) => {
                let state = serde_json::from_str(&data)?;
                Ok(Some(state))
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(PersistError::Io(e)),
        }
    }

    /// Load state, falling back to a default if the file doesn't exist.
    pub fn load_or_default<T: serde::de::DeserializeOwned + Default>(
        &self,
    ) -> Result<T, PersistError> {
        self.load_or_none().map(|opt| opt.unwrap_or_default())
    }

    /// Remove the persisted state file.
    pub fn clear(&self) -> Result<(), PersistError> {
        match std::fs::remove_file(&self.path) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(PersistError::Io(e)),
        }
    }

    /// Check if a persisted state file exists.
    pub fn exists(&self) -> bool {
        self.path.exists()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestState {
        count: i32,
        name: String,
    }

    #[test]
    fn round_trip() {
        let dir = std::env::temp_dir().join("dewey_persist_test");
        let store = StateStore::with_path(dir.join("test_state.json"));
        let state = TestState {
            count: 42,
            name: "hello".into(),
        };
        store.save(&state).unwrap();
        let loaded: TestState = store.load().unwrap();
        assert_eq!(state, loaded);
        store.clear().unwrap();
    }

    #[test]
    fn load_or_none_missing() {
        let store = StateStore::with_path("/tmp/dewey_nonexistent_persist_test_12345.json");
        let result: Result<Option<TestState>, _> = store.load_or_none();
        assert!(result.unwrap().is_none());
    }
}
