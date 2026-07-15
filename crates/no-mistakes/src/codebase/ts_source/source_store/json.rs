use std::io;
use std::sync::Arc;

/// Cached failure while loading a JSON document.
#[derive(Debug, Clone)]
#[doc(hidden)]
pub enum JsonLoadError {
    Io(Arc<io::Error>),
    Syntax(Arc<serde_json::Error>),
}

impl std::fmt::Display for JsonLoadError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "{error}"),
            Self::Syntax(error) => write!(formatter, "{error}"),
        }
    }
}

impl std::error::Error for JsonLoadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error.as_ref()),
            Self::Syntax(error) => Some(error.as_ref()),
        }
    }
}

#[doc(hidden)]
pub type JsonParseOutcome = Result<Arc<serde_json::Value>, JsonLoadError>;
