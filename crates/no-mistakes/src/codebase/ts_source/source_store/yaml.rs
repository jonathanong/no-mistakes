use serde::Deserialize;
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
        let cached = self.load_yaml(path);
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
        let path = crate::codebase::ts_source::normalize_discovery_path(path);
        let slots = self.yaml_documents.get_or_init(dashmap::DashMap::new);
        let cell = super::once_lock_slot(slots, path.clone());
        self.increment("yaml.requests", 1);
        let parsed = std::cell::Cell::new(false);
        cell.get_or_init(|| {
            parsed.set(true);
            self.yaml_parse_count.fetch_add(1, Ordering::Relaxed);
            self.increment("yaml.parses", 1);
            match self.read_path(&path) {
                Ok(source) => cached_from_str(self, &source),
                Err(error) => {
                    self.increment("yaml.errors", 1);
                    CachedYaml::failure(YamlLoadError::Io(error))
                }
            }
        });
        if !parsed.get() {
            self.increment("yaml.cache_hits", 1);
        }
        cell
    }
}

fn cached_from_str(store: &super::SourceStore, source: &str) -> CachedYaml {
    let split = split_documents(source);
    let documents = document_view(&split);
    let single = single_view(&split);
    if documents.is_err() {
        store.increment("yaml.errors", 1);
    }
    CachedYaml::from_views(documents, single)
}

struct SplitYaml {
    documents: Vec<Arc<serde_yaml::Value>>,
    failure: Option<Arc<serde_yaml::Error>>,
}

fn split_documents(source: &str) -> SplitYaml {
    let mut documents = Vec::new();
    let mut failure = None;
    for document in serde_yaml::Deserializer::from_str(source) {
        match serde_yaml::Value::deserialize(document) {
            Ok(value) => documents.push(Arc::new(value)),
            Err(error) => {
                failure = Some(Arc::new(error));
                break;
            }
        }
    }
    SplitYaml { documents, failure }
}

fn document_view(split: &SplitYaml) -> YamlDocumentsOutcome {
    if split.documents.is_empty() {
        if let Some(error) = &split.failure {
            return Err(YamlLoadError::Syntax(Arc::clone(error)));
        }
    }
    Ok(Arc::from(split.documents.as_slice()))
}

/// `from_str` keeps one clean document. No documents is an empty value.
/// A later document, including one that does not parse, is rejected.
fn single_view(split: &SplitYaml) -> YamlParseOutcome {
    if split.documents.len() == 1 && split.failure.is_none() {
        return Ok(Arc::clone(&split.documents[0]));
    }
    if split.documents.is_empty() {
        if let Some(error) = &split.failure {
            return Err(YamlLoadError::Syntax(Arc::clone(error)));
        }
        return Ok(null_document());
    }
    Err(YamlLoadError::Syntax(extra_document_error()))
}

fn null_document() -> Arc<serde_yaml::Value> {
    static CELL: OnceLock<Arc<serde_yaml::Value>> = OnceLock::new();
    Arc::clone(CELL.get_or_init(|| Arc::new(serde_yaml::Value::Null)))
}

fn extra_document_error() -> Arc<serde_yaml::Error> {
    static CELL: OnceLock<Arc<serde_yaml::Error>> = OnceLock::new();
    Arc::clone(CELL.get_or_init(|| {
        Arc::new(
            serde_yaml::from_str::<serde_yaml::Value>("a: 1\n---\nb: 2\n")
                .expect_err("two YAML documents are rejected"),
        )
    }))
}

#[cfg(test)]
#[path = "yaml_tests.rs"]
mod tests;
