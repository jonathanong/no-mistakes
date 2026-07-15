use super::FileInventory;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};

pub(super) type ValidatedPathCache = HashMap<(PathBuf, PathBuf), Arc<OnceLock<Option<PathBuf>>>>;

pub(super) fn validated_regular_path(
    inventory: &FileInventory,
    validations: &Mutex<ValidatedPathCache>,
    root: &Path,
    candidate: &Path,
) -> Option<PathBuf> {
    if inventory
        .classification_for_path(candidate)
        .is_some_and(super::super::FileClassification::is_lexical_file)
    {
        return Some(candidate.to_path_buf());
    }
    let key = (root.to_path_buf(), candidate.to_path_buf());
    let cell = {
        let mut validations = validations
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
