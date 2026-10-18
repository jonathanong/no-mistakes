//! Completes executor SQL that is built from imported helpers.
//!
//! The per-file pass recorded each such call with a private hole where the
//! helper's SQL goes, and summarized every helper it declares. Here the
//! request resolver names the module behind each imported helper, re-exports
//! are followed, and the helper's summary fills the hole. Nothing is read or
//! parsed again, and an unproven link leaves the call `Dynamic`.

use super::CheckFileFacts;
use crate::codebase::postgres::{
    splice_imported_sql, EmbeddedSqlCall, EmbeddedSqlOptions, PendingImportedSql,
};
use crate::codebase::ts_source::FileIdMap;
use crate::fx::FxHashMap;
use locate::Locator;
use program::Shared;
use std::path::{Path, PathBuf};

mod locate;
mod program;

#[cfg(test)]
mod tests;

pub(super) fn has_candidates(files: &FileIdMap<CheckFileFacts>) -> bool {
    files.into_iter().any(|(_, file)| {
        file.embedded_sql
            .iter()
            .any(|(_, embedded)| !embedded.pending_imported.is_empty())
    })
}

/// A completed call and where it goes: file, profile index, original span.
type Completed = (PathBuf, usize, (u32, u32), EmbeddedSqlCall, Vec<u32>);

pub(super) fn project(
    files: &mut FileIdMap<CheckFileFacts>,
    resolve: impl FnMut(&str, &Path) -> Option<PathBuf>,
) {
    let completed = complete(files, resolve);
    for (path, profile, span, call, lines) in completed {
        if let Some(file) = files.get_mut(&path) {
            file.embedded_sql[profile]
                .1
                .apply_imported_sql(span, call, lines);
        }
    }
}

fn complete(
    files: &FileIdMap<CheckFileFacts>,
    resolve: impl FnMut(&str, &Path) -> Option<PathBuf>,
) -> Vec<Completed> {
    let mut shared = Shared::new(files, Locator::new(files, resolve));
    let mut completed = Vec::new();
    for (path, file) in files {
        for (profile, (options, embedded)) in file.embedded_sql.iter().enumerate() {
            for pending in &embedded.pending_imported {
                if let Some(call) = complete_call(&mut shared, path, profile, options, pending) {
                    let lines = pending.sites.iter().map(|(_, site)| site.line).collect();
                    completed.push((path.clone(), profile, pending.span, call, lines));
                }
            }
        }
    }
    completed
}

fn complete_call(
    shared: &mut Shared<'_, impl FnMut(&str, &Path) -> Option<PathBuf>>,
    path: &Path,
    profile: usize,
    options: &EmbeddedSqlOptions,
    pending: &PendingImportedSql,
) -> Option<EmbeddedSqlCall> {
    let mut holes = FxHashMap::default();
    for (index, site) in &pending.sites {
        holes.insert(*index, shared.fill(profile, options, path, site)?);
    }
    splice_imported_sql(&pending.template, &holes)
}
