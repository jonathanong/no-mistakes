use std::io;
use std::path::Path;
use std::sync::Arc;

/// Memoized result of a source read.
pub type SourceReadOutcome = Result<Arc<str>, Arc<io::Error>>;

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

/// Borrowed request-scoped source storage for TypeScript analysis consumers.
pub trait SourceAccess: Send + Sync {
    fn read_path(&self, path: &Path) -> SourceReadOutcome;
    fn read_source_path(&self, path: &Path) -> SourceReadOutcome;
    fn parse_json_path(&self, path: &Path) -> JsonParseOutcome;
}

impl<S: SourceAccess + ?Sized> SourceAccess for Arc<S> {
    fn read_path(&self, path: &Path) -> SourceReadOutcome {
        self.as_ref().read_path(path)
    }

    fn read_source_path(&self, path: &Path) -> SourceReadOutcome {
        self.as_ref().read_source_path(path)
    }

    fn parse_json_path(&self, path: &Path) -> JsonParseOutcome {
        self.as_ref().parse_json_path(path)
    }
}

/// Read through a prepared store, or directly from disk when compatibility callers omit one.
pub fn read_prepared_or_open(sources: Option<&dyn SourceAccess>, path: &Path) -> SourceReadOutcome {
    let indexable = no_mistakes_ts_facts::extract::is_indexable(path);
    match sources {
        Some(store) if indexable => store.read_source_path(path),
        Some(store) => store.read_path(path),
        None if indexable => read_lossy_arc(path).map_err(Arc::new),
        None => read_utf8_arc(path).map_err(Arc::new),
    }
}

fn read_utf8_arc(path: &Path) -> io::Result<Arc<str>> {
    let bytes = std::fs::read(path)?;
    let string = String::from_utf8(bytes)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    Ok(Arc::from(string))
}

fn read_lossy_arc(path: &Path) -> io::Result<Arc<str>> {
    let bytes = std::fs::read(path)?;
    match String::from_utf8(bytes) {
        Ok(text) => Ok(Arc::from(text)),
        Err(error) => Ok(Arc::from(
            String::from_utf8_lossy(error.as_bytes()).as_ref(),
        )),
    }
}

#[cfg(test)]
mod tests;
