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

impl super::SourceStore {
    #[doc(hidden)]
    pub fn parse_json_path(&self, path: &std::path::Path) -> JsonParseOutcome {
        let path = crate::codebase::ts_source::normalize_discovery_path(path);
        let cell = super::once_lock_slot(&self.json_parses, path.clone());
        self.increment("manifest.requests", 1);
        let parsed = std::cell::Cell::new(false);
        let result = cell
            .get_or_init(|| {
                parsed.set(true);
                self.json_parse_count
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                self.increment("manifest.parses", 1);
                match self.read_path(&path) {
                    Ok(source) => serde_json::from_str(&source)
                        .map(Arc::new)
                        .map_err(|error| {
                            self.increment("manifest.errors", 1);
                            JsonLoadError::Syntax(Arc::new(error))
                        }),
                    Err(error) => {
                        self.increment("manifest.errors", 1);
                        Err(JsonLoadError::Io(error))
                    }
                }
            })
            .clone();
        if !parsed.get() {
            self.increment("manifest.cache_hits", 1);
        }
        result
    }

    #[doc(hidden)]
    pub fn json_parse_count(&self) -> usize {
        self.json_parse_count
            .load(std::sync::atomic::Ordering::Relaxed)
    }
}
