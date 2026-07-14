use super::{FileId, FileInventory};
use std::io;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, OnceLock};

/// Memoized result of a strict UTF-8 source read.
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

type ValidatedPathCache = std::collections::HashMap<
    (std::path::PathBuf, std::path::PathBuf),
    Arc<OnceLock<Option<std::path::PathBuf>>>,
>;

/// Lazy request-scoped source storage for a frozen file inventory.
///
/// Each logical file is read at most once. Successful text and failures are
/// both retained until the request is dropped.
#[doc(hidden)]
pub struct SourceStore {
    inventory: Arc<FileInventory>,
    reads: Vec<OnceLock<SourceReadOutcome>>,
    json_parses: std::sync::Mutex<
        std::collections::HashMap<std::path::PathBuf, Arc<OnceLock<JsonParseOutcome>>>,
    >,
    supplemental_reads: std::sync::Mutex<
        std::collections::HashMap<std::path::PathBuf, Arc<OnceLock<SourceReadOutcome>>>,
    >,
    validated_regular_paths: std::sync::Mutex<ValidatedPathCache>,
    physical_reads: AtomicUsize,
    json_parse_count: AtomicUsize,
}

impl SourceStore {
    #[doc(hidden)]
    pub fn new(inventory: Arc<FileInventory>) -> Self {
        let reads = (0..inventory.len()).map(|_| OnceLock::new()).collect();
        Self {
            inventory,
            reads,
            json_parses: std::sync::Mutex::new(std::collections::HashMap::new()),
            supplemental_reads: std::sync::Mutex::new(std::collections::HashMap::new()),
            validated_regular_paths: std::sync::Mutex::new(std::collections::HashMap::new()),
            physical_reads: AtomicUsize::new(0),
            json_parse_count: AtomicUsize::new(0),
        }
    }

    #[doc(hidden)]
    pub fn inventory(&self) -> &Arc<FileInventory> {
        &self.inventory
    }

    #[doc(hidden)]
    pub fn read(&self, id: FileId) -> Option<SourceReadOutcome> {
        let path = self.inventory.path(id)?;
        let slot = self.reads.get(id.index())?;
        Some(
            slot.get_or_init(|| {
                self.physical_reads.fetch_add(1, Ordering::Relaxed);
                std::fs::read_to_string(path)
                    .map(Arc::<str>::from)
                    .map_err(Arc::new)
            })
            .clone(),
        )
    }

    #[doc(hidden)]
    pub fn read_path(&self, path: &Path) -> SourceReadOutcome {
        if let Some(id) = self.inventory.id_for_normalized_path(path) {
            return self
                .read(id)
                .expect("inventory IDs always resolve to their source slot");
        }
        let path = super::normalize_discovery_path(path);
        if let Some(id) = self.inventory.id_for_normalized_path(&path) {
            return self
                .read(id)
                .expect("inventory IDs always resolve to their source slot");
        }
        let cell = {
            let mut reads = self
                .supplemental_reads
                .lock()
                .expect("supplemental source-store reads mutex poisoned");
            Arc::clone(
                reads
                    .entry(path.clone())
                    .or_insert_with(|| Arc::new(OnceLock::new())),
            )
        };
        cell.get_or_init(|| {
            self.physical_reads.fetch_add(1, Ordering::Relaxed);
            std::fs::read_to_string(&path)
                .map(Arc::<str>::from)
                .map_err(Arc::new)
        })
        .clone()
    }

    #[doc(hidden)]
    pub fn parse_json_path(&self, path: &Path) -> JsonParseOutcome {
        let path = super::normalize_discovery_path(path);
        let cell = {
            let mut parses = self
                .json_parses
                .lock()
                .expect("JSON parse cache mutex poisoned");
            Arc::clone(
                parses
                    .entry(path.clone())
                    .or_insert_with(|| Arc::new(OnceLock::new())),
            )
        };
        cell.get_or_init(|| {
            self.json_parse_count.fetch_add(1, Ordering::Relaxed);
            match self.read_path(&path) {
                Ok(source) => serde_json::from_str(&source)
                    .map(Arc::new)
                    .map_err(|error| JsonLoadError::Syntax(Arc::new(error))),
                Err(error) => Err(JsonLoadError::Io(error)),
            }
        })
        .clone()
    }

    /// Validate a lexical candidate once for suppression consumers. Regular
    /// inventory entries were already classified at discovery; symlinks and
    /// supplemental paths additionally require canonical containment.
    pub(crate) fn validated_regular_path(
        &self,
        root: &Path,
        candidate: &Path,
    ) -> Option<std::path::PathBuf> {
        if self
            .inventory
            .classification_for_path(candidate)
            .is_some_and(super::FileClassification::is_lexical_file)
        {
            return Some(candidate.to_path_buf());
        }
        let key = (root.to_path_buf(), candidate.to_path_buf());
        let cell = {
            let mut validations = self
                .validated_regular_paths
                .lock()
                .expect("source path validation mutex poisoned");
            Arc::clone(
                validations
                    .entry(key.clone())
                    .or_insert_with(|| Arc::new(OnceLock::new())),
            )
        };
        cell.get_or_init(|| {
            let canonical_root = std::fs::canonicalize(&key.0).ok()?;
            let canonical_candidate = std::fs::canonicalize(&key.1).ok()?;
            let metadata = std::fs::metadata(&canonical_candidate).ok()?;
            (canonical_candidate.starts_with(canonical_root) && metadata.is_file())
                .then_some(key.1.clone())
        })
        .clone()
    }

    #[doc(hidden)]
    pub fn json_parse_count(&self) -> usize {
        self.json_parse_count.load(Ordering::Relaxed)
    }

    pub fn physical_read_count(&self) -> usize {
        self.physical_reads.load(Ordering::Relaxed)
    }
}

#[cfg(test)]
mod tests;
