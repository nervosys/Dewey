//! Error types for the Dewey framework.
//!
//! Provides a unified [`Error`] enum covering all failure modes in the
//! framework: I/O errors, protocol errors, action dispatch errors, and
//! layout constraint violations.

use std::fmt;

/// The unified error type for all Dewey operations.
///
/// Each variant wraps either an upstream error or a descriptive message.
/// Match on variants to decide recovery strategy (retry, log, display to user).
#[derive(Debug)]
pub enum Error {
    /// An I/O error from the backend or file system.
    ///
    /// Wraps [`std::io::Error`]. Common causes: missing files, permission
    /// denied, broken pipes on agent stdin/stdout. Check the inner
    /// [`ErrorKind`](std::io::ErrorKind) for retry-ability.
    Io(std::io::Error),
    /// A JSON serialization or deserialization error.
    ///
    /// Wraps [`serde_json::Error`]. Typically caused by malformed agent
    /// protocol messages or corrupt persistence files. Inspect the inner
    /// error's `line()` / `column()` for diagnostics.
    Json(serde_json::Error),
    /// An agent protocol error (malformed request, unknown message type, etc.).
    ///
    /// Returned when an agent sends a message that cannot be parsed or
    /// references an unsupported protocol version.
    Protocol(String),
    /// An action dispatch error (unknown action, validation failure, etc.).
    ///
    /// Returned when an ontology action is invoked with invalid arguments
    /// or targets a widget that does not support the requested action.
    Action(String),
    /// A widget was not found by its agent ID.
    ///
    /// The contained string is the agent ID that failed to resolve.
    /// Ensure the widget was created with `.agent_id()` and is currently
    /// mounted in the view tree.
    WidgetNotFound(String),
    /// A layout constraint error.
    ///
    /// Returned when the layout engine cannot satisfy constraints—e.g.,
    /// negative sizes, circular dependencies, or over-constrained flex.
    Layout(String),
    /// A rendering / backend error.
    ///
    /// Returned when the GPU backend, surface, or painter encounters a
    /// fatal error such as a lost device or incompatible surface format.
    Render(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(e) => write!(f, "I/O error: {e}"),
            Error::Json(e) => write!(f, "JSON error: {e}"),
            Error::Protocol(msg) => write!(f, "Protocol error: {msg}"),
            Error::Action(msg) => write!(f, "Action error: {msg}"),
            Error::WidgetNotFound(id) => write!(f, "Widget not found: {id}"),
            Error::Layout(msg) => write!(f, "Layout error: {msg}"),
            Error::Render(msg) => write!(f, "Render error: {msg}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io(e) => Some(e),
            Error::Json(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::Json(e)
    }
}

/// A specialized [`Result`] type for Dewey operations.
pub type Result<T> = std::result::Result<T, Error>;
