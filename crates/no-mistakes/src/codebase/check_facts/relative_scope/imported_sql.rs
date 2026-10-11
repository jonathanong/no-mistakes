use super::CheckFileFacts;
use crate::codebase::postgres::{EmbeddedSqlOptions, RecoveredSql};
use crate::codebase::ts_source::FileIdMap;
use crate::fx::FxHashMap;
use std::path::{Path, PathBuf};

type ExportKey = (PathBuf, EmbeddedSqlOptions, String);

pub(super) fn has_candidates(files: &FileIdMap<CheckFileFacts>) -> bool {
    files.into_iter().any(|(_, file)| {
        file.embedded_sql
            .iter()
            .any(|(_, embedded)| !embedded.pending_imported_calls.is_empty())
    })
}

/// Reuse the request resolver and per-file summaries; no source is reopened or reparsed.
pub(super) fn project(
    files: &mut FileIdMap<CheckFileFacts>,
    mut resolve: impl FnMut(&str, &Path) -> Option<PathBuf>,
) {
    let mut exports: FxHashMap<ExportKey, RecoveredSql> = FxHashMap::default();
    for (path, file) in &*files {
        for binding in &file.ts.exported_bindings {
            if binding.specifier.is_some() {
                continue;
            }
            for (profile, embedded) in &file.embedded_sql {
                if let Some(sql) = embedded.local_helpers.get(&binding.local) {
                    exports.insert(
                        (path.clone(), profile.clone(), binding.exported.clone()),
                        sql.clone().with_source_file(path),
                    );
                }
            }
        }
    }
    for (path, file) in &mut *files {
        let imported: FxHashMap<_, _> = file
            .ts
            .imported_bindings
            .iter()
            .filter(|binding| !binding.is_type_only)
            .map(|binding| (binding.local.as_str(), binding))
            .collect();
        for (profile, embedded) in &mut file.embedded_sql {
            for pending in embedded.pending_imported_calls.clone() {
                let Some(binding) = imported.get(pending.local.as_str()) else {
                    continue;
                };
                let Some(target) = resolve(&binding.specifier, path) else {
                    continue;
                };
                if let Some(sql) = exports.get(&(target, profile.clone(), binding.imported.clone()))
                {
                    embedded.apply_imported_helper(pending.span, sql.clone());
                }
            }
        }
    }
}
