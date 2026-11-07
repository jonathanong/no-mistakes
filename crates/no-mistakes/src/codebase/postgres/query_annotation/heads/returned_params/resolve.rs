//! Approve every direct helper arm using the canonical resolver projection.
use super::Pending;
use crate::codebase::{
    check_facts::{
        sql_source_prerequisites::{source_failed, FailedSqlSources},
        CheckFileFacts,
    },
    ts_source::FileIdMap,
};
use std::path::{Path, PathBuf};

pub(crate) fn project(
    files: &FileIdMap<CheckFileFacts>,
    resolve: &impl Fn(&str, &Path) -> Option<PathBuf>,
    path: &Path,
    pending: &Pending,
) -> (bool, FailedSqlSources) {
    let mut proven = true;
    let mut failed = FailedSqlSources::default();
    for (specifier, imported) in &pending.imports {
        let Some(target) = resolve(specifier, path) else {
            proven = false;
            continue;
        };
        if source_failed(files, &target) {
            failed.insert(target);
            proven = false;
            continue;
        }
        proven &= files
            .get(&target)
            .and_then(|file| file.query_annotation_heads.as_deref())
            .is_some_and(|heads| heads.exported_returned_params.contains(imported));
    }
    (proven, failed)
}
