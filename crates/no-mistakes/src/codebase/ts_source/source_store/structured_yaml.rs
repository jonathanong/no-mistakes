use super::{once_lock_slot, SourceStore};
use crate::codebase::structured_value::{parse_yaml_documents, YamlDocuments};
use dashmap::DashMap;
use std::io;
use std::path::{Path, PathBuf};
#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, OnceLock};

type StructuredYamlOutcome = Result<(Arc<str>, Arc<YamlDocuments>), Arc<io::Error>>;
type StructuredYamlSlot = Arc<OnceLock<StructuredYamlOutcome>>;

#[derive(Default)]
pub(super) struct StructuredYamlSlots {
    paths: DashMap<PathBuf, StructuredYamlSlot>,
    #[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
    initializations: AtomicUsize,
}

impl SourceStore {
    pub(crate) fn parse_structured_yaml_path(&self, path: &Path) -> StructuredYamlOutcome {
        let path = crate::codebase::ts_source::normalize_discovery_path(path);
        let slots = self
            .structured_yaml
            .get_or_init(StructuredYamlSlots::default);
        let slot = once_lock_slot(&slots.paths, path.clone());
        slot.get_or_init(|| {
            #[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
            slots.initializations.fetch_add(1, Ordering::Relaxed);
            let source = self.read_path(&path).map_err(strict_yaml_read_error)?;
            let documents = Arc::new(parse_yaml_documents(&source));
            Ok((source, documents))
        })
        .clone()
    }

    #[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
    pub(crate) fn structured_yaml_initialization_count(&self) -> usize {
        self.structured_yaml
            .get()
            .map_or(0, |slots| slots.initializations.load(Ordering::Relaxed))
    }
}

fn strict_yaml_read_error(error: Arc<io::Error>) -> Arc<io::Error> {
    // Preserve read_to_string's diagnostic while the shared read retains the invalid bytes.
    if error
        .get_ref()
        .is_some_and(|cause| cause.is::<std::string::FromUtf8Error>())
    {
        return Arc::new(io::Error::new(
            io::ErrorKind::InvalidData,
            "stream did not contain valid UTF-8",
        ));
    }
    error
}
