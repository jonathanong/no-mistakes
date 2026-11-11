use super::{SourceReadOutcome, SourceStore};
use dashmap::DashMap;
use std::hash::Hash;
use std::io;
use std::path::Path;
use std::sync::{Arc, OnceLock};

/// Retain invalid bytes in the memoized failure for strict consumers.
pub(super) fn read_utf8_arc(path: &Path) -> io::Result<Arc<str>> {
    let bytes = std::fs::read(path)?;
    let string = String::from_utf8(bytes)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    Ok(Arc::from(string))
}

impl SourceStore {
    /// Decode language source lossily from the same memoized physical read.
    #[doc(hidden)]
    pub fn read_source_path(&self, path: &Path) -> SourceReadOutcome {
        let error = match self.read_path(path) {
            Ok(source) => return Ok(source),
            Err(error) => error,
        };
        let Some(invalid) = error
            .get_ref()
            .and_then(|error| error.downcast_ref::<std::string::FromUtf8Error>())
        else {
            return Err(error);
        };
        let path = super::super::normalize_discovery_path(path);
        let slot = once_lock_slot(&self.source_projections, path);
        slot.get_or_init(|| {
            Ok(Arc::from(
                String::from_utf8_lossy(invalid.as_bytes()).as_ref(),
            ))
        })
        .clone()
    }
}

pub(super) fn once_lock_slot<K, T>(map: &DashMap<K, Arc<OnceLock<T>>>, key: K) -> Arc<OnceLock<T>>
where
    K: Eq + Hash,
{
    if let Some(existing) = map.get(&key) {
        return Arc::clone(existing.value());
    }
    Arc::clone(
        map.entry(key)
            .or_insert_with(|| Arc::new(OnceLock::new()))
            .value(),
    )
}
