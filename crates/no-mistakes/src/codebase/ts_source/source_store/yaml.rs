mod parse;
use parse::cached_from_str;
use std::io;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock};

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

    pub(crate) fn line(&self) -> usize {
        match self {
            Self::Io(_) => 1,
            Self::Syntax(error) => error.location().map_or(1, |location| location.line()),
        }
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

/// Every YAML document in one file that parsed. A later broken document does
/// not drop the documents before it. An empty failure is a syntax error.
#[doc(hidden)]
pub type YamlDocumentsOutcome = Result<Arc<[Arc<serde_yaml::Value>]>, YamlLoadError>;

#[derive(Clone, Copy, Default, Eq, Hash, PartialEq)]
pub(super) struct YamlProjection {
    removed_bom: bool,
    replaced_invalid_bytes: bool,
}

/// One document walk. The strict single-document view is derived from it.
pub(super) struct CachedYaml {
    documents: YamlDocumentsOutcome,
    single: YamlParseOutcome,
    /// The document walk succeeded and the single-document view did not.
    strict_rejected: bool,
    strict_error_counted: AtomicBool,
}

impl CachedYaml {
    fn from_views(documents: YamlDocumentsOutcome, single: YamlParseOutcome) -> Self {
        Self {
            strict_rejected: single.is_err() && documents.is_ok(),
            documents,
            single,
            strict_error_counted: AtomicBool::new(false),
        }
    }

    fn failure(error: YamlLoadError) -> Self {
        Self::from_views(Err(error.clone()), Err(error))
    }

    fn note_strict_rejection(&self, store: &super::SourceStore) {
        if self.strict_rejected && !self.strict_error_counted.swap(true, Ordering::Relaxed) {
            store.increment("yaml.errors", 1);
        }
    }
}

impl super::SourceStore {
    #[doc(hidden)]
    pub fn parse_yaml_path(&self, path: &Path) -> YamlParseOutcome {
        self.parse_single_yaml(self.load_yaml(path))
    }

    /// Reuse the YAML parse for a strict or lossy read from this store.
    pub(crate) fn parse_loaded_yaml(
        &self,
        path: &Path,
        source: &str,
        trim_bom: bool,
    ) -> YamlParseOutcome {
        let text = if trim_bom {
            source.trim_start_matches('\u{feff}')
        } else {
            source
        };
        let projection = YamlProjection {
            removed_bom: text.len() != source.len(),
            replaced_invalid_bytes: source.contains('\u{fffd}') && self.read_path(path).is_err(),
        };
        let cached = self.load_yaml_projection(path, projection, |_| cached_from_str(self, text));
        self.parse_single_yaml(cached)
    }

    fn parse_single_yaml(&self, cached: Arc<OnceLock<CachedYaml>>) -> YamlParseOutcome {
        let entry = cached.get().expect("yaml cache is initialized");
        entry.note_strict_rejection(self);
        entry.single.clone()
    }

    /// Documents from the same cached parse as [`Self::parse_yaml_path`].
    /// The single-document view rejects an empty, partial, or multi-document
    /// stream. A later call does not deserialize the file again.
    #[doc(hidden)]
    pub fn parse_yaml_documents_path(&self, path: &Path) -> YamlDocumentsOutcome {
        self.load_yaml(path)
            .get()
            .expect("yaml cache is initialized")
            .documents
            .clone()
    }

    #[doc(hidden)]
    pub fn yaml_parse_count(&self) -> usize {
        self.yaml_parse_count
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    fn load_yaml(&self, path: &Path) -> Arc<OnceLock<CachedYaml>> {
        self.load_yaml_projection(path, YamlProjection::default(), |path| {
            match self.read_path(path) {
                Ok(source) => cached_from_str(self, &source),
                Err(error) => {
                    self.increment("yaml.errors", 1);
                    CachedYaml::failure(YamlLoadError::Io(error))
                }
            }
        })
    }

    fn load_yaml_projection(
        &self,
        path: &Path,
        projection: YamlProjection,
        load: impl FnOnce(&Path) -> CachedYaml,
    ) -> Arc<OnceLock<CachedYaml>> {
        let path = crate::codebase::ts_source::normalize_discovery_path(path);
        let slots = self.yaml_documents.get_or_init(dashmap::DashMap::new);
        let cell = super::once_lock_slot(slots, (path.clone(), projection));
        self.increment("yaml.requests", 1);
        let parsed = std::cell::Cell::new(false);
        cell.get_or_init(|| {
            parsed.set(true);
            self.yaml_parse_count.fetch_add(1, Ordering::Relaxed);
            self.increment("yaml.parses", 1);
            load(&path)
        });
        if !parsed.get() {
            self.increment("yaml.cache_hits", 1);
        }
        cell
    }
}

#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
mod tests;
