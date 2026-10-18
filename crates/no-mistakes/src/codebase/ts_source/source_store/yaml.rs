use std::io;
use std::path::Path;
use std::sync::Arc;

/// Cached failure while loading a YAML document.
#[derive(Debug, Clone)]
#[doc(hidden)]
pub enum YamlLoadError {
    Io(Arc<io::Error>),
    Syntax(Arc<serde_yaml::Error>),
}

impl YamlLoadError {
    #[doc(hidden)]
    pub fn is_io(&self) -> bool {
        matches!(self, Self::Io(_))
    }
}

impl std::fmt::Display for YamlLoadError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "{error}"),
            Self::Syntax(error) => write!(formatter, "{error}"),
        }
    }
}

impl std::error::Error for YamlLoadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error.as_ref()),
            Self::Syntax(error) => Some(error.as_ref()),
        }
    }
}

#[doc(hidden)]
pub type YamlParseOutcome = Result<Arc<serde_yaml::Value>, YamlLoadError>;

impl super::SourceStore {
    #[doc(hidden)]
    pub fn parse_yaml_path(&self, path: &Path) -> YamlParseOutcome {
        let path = crate::codebase::ts_source::normalize_discovery_path(path);
        let slots = self.yaml_parses.get_or_init(dashmap::DashMap::new);
        let cell = super::once_lock_slot(slots, path.clone());
        self.increment("yaml.requests", 1);
        let parsed = std::cell::Cell::new(false);
        let result = cell
            .get_or_init(|| {
                parsed.set(true);
                self.yaml_parse_count
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                self.increment("yaml.parses", 1);
                match self.read_path(&path) {
                    Ok(source) => serde_yaml::from_str(&source)
                        .map(Arc::new)
                        .map_err(|error| {
                            self.increment("yaml.errors", 1);
                            YamlLoadError::Syntax(Arc::new(error))
                        }),
                    Err(error) => {
                        self.increment("yaml.errors", 1);
                        Err(YamlLoadError::Io(error))
                    }
                }
            })
            .clone();
        if !parsed.get() {
            self.increment("yaml.cache_hits", 1);
        }
        result
    }

    #[doc(hidden)]
    pub fn yaml_parse_count(&self) -> usize {
        self.yaml_parse_count
            .load(std::sync::atomic::Ordering::Relaxed)
    }
}

#[cfg(test)]
#[path = "yaml_tests.rs"]
mod tests;
