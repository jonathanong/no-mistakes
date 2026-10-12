use dashmap::DashMap;
use rustc_hash::FxBuildHasher;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};

#[cfg(test)]
mod tests;

pub(super) type ParsedImporterResult = Result<ParsedImporters, String>;
type ParsedImporterCell = Arc<OnceLock<Arc<ParsedImporterResult>>>;

pub(in crate::codebase::rules::forbidden_workspace_closure) struct LockfileImporterCache {
    entries: DashMap<PathBuf, ParsedImporterCell, FxBuildHasher>,
    observer: Option<Arc<crate::diagnostics::InvocationObserver>>,
}

pub(super) enum ParsedImporters {
    Malformed,
    Loaded {
        env_prefix_valid: bool,
        by_path: BTreeMap<String, crate::codebase::lockfile::pnpm::PnpmImporter>,
    },
}

impl LockfileImporterCache {
    pub(in crate::codebase::rules::forbidden_workspace_closure) fn with_observer(
        observer: Option<Arc<crate::diagnostics::InvocationObserver>>,
    ) -> Self {
        Self {
            entries: DashMap::with_hasher(FxBuildHasher),
            observer,
        }
    }

    pub(super) fn get_or_compute(
        &self,
        path: &Path,
        compute: impl FnOnce() -> ParsedImporterResult,
    ) -> Arc<ParsedImporterResult> {
        let entry = self
            .entries
            .entry(path.to_path_buf())
            .or_insert_with(|| Arc::new(OnceLock::new()));
        let cell = Arc::clone(&entry);
        drop(entry);
        cell.get_or_init(|| {
            if let Some(observer) = &self.observer {
                observer.increment(
                    "forbidden_workspace_closure.lockfile_importer_constructions",
                    1,
                );
            }
            Arc::new(compute())
        })
        .clone()
    }
}

pub(super) fn parse_importer_content(content: &str) -> ParsedImporterResult {
    let Ok(docs) = crate::codebase::lockfile::pnpm::load_documents(content) else {
        return Ok(ParsedImporters::Malformed);
    };
    let by_path = docs
        .importers()
        .into_iter()
        .map(|importer| (super::normalize_importer_path(&importer.path), importer))
        .collect();
    Ok(ParsedImporters::Loaded {
        env_prefix_valid: crate::codebase::lockfile::pnpm::validate_env_prefix(&docs).is_ok(),
        by_path,
    })
}
