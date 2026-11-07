//! Completes executor SQL that is built from imported helpers.
//!
//! The per-file pass recorded each such call with a private hole where the
//! helper's SQL goes, and summarized every helper it declares. Here the
//! request resolver names the module behind each imported helper, re-exports
//! are followed, and the helper's summary fills the hole. Nothing is read or
//! parsed again, and an unproven link leaves the call `Dynamic`.

use super::CheckFileFacts;
use crate::codebase::check_facts::sql_source_prerequisites::{
    SqlProjection, SqlSourceKey, SqlSourcePrerequisites,
};
use crate::codebase::postgres::{
    splice_imported_sql, EmbeddedSqlCall, EmbeddedSqlOptions, PendingImportedSql,
};
use crate::codebase::ts_source::FileIdMap;
use crate::fx::FxHashMap;
use locate::Locator;
use program::Shared;
use std::path::{Path, PathBuf};

mod constants;
mod locate;
mod lock_helpers;
mod program;

#[cfg(test)]
mod tests;

pub(super) fn has_candidates(files: &FileIdMap<CheckFileFacts>) -> bool {
    files.into_iter().any(|(_, file)| {
        file.embedded_sql.iter().any(|(_, embedded)| {
            !embedded.pending_imported.is_empty() || !embedded.imported_lock_helpers.is_empty()
        })
    })
}

/// A completed call and where it goes: file, profile index, original span.
type Completed = (PathBuf, usize, (u32, u32), EmbeddedSqlCall, Vec<u32>);
type LockCompleted = (PathBuf, usize, (u32, u32), Vec<EmbeddedSqlCall>);

pub(super) fn project(
    files: &mut FileIdMap<CheckFileFacts>,
    resolve: impl FnMut(&str, &Path) -> Option<PathBuf>,
) {
    let (completed, prerequisites, locks) = complete(files, resolve);
    for (path, profile, span, calls) in locks {
        let embedded = &mut files.get_mut(&path).unwrap().embedded_sql[profile].1;
        for call in calls {
            embedded.calls.push(call);
            embedded.call_spans.push(span);
        }
        embedded.imported_lock_helpers.clear();
    }
    for (path, prerequisite) in prerequisites {
        if let Some(file) = files.get_mut(&path) {
            file.sql_source_prerequisites.merge(prerequisite);
        }
    }
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
) -> (
    Vec<Completed>,
    Vec<(PathBuf, SqlSourcePrerequisites)>,
    Vec<LockCompleted>,
) {
    let mut shared = Shared::new(files, Locator::new(files, resolve));
    let mut completed = Vec::new();
    let mut prerequisites = Vec::new();
    let mut locks = Vec::new();
    for (path, file) in files {
        let mut provenance = SqlSourcePrerequisites::default();
        for (profile, (options, embedded)) in file.embedded_sql.iter().enumerate() {
            for invocation in &embedded.imported_lock_helpers {
                let calls = lock_helpers::complete(&mut shared, path, options, invocation);
                locks.push((path.clone(), profile, invocation.span, calls));
            }
            for pending in &embedded.pending_imported {
                shared.failures.clear();
                let call = complete_call(&mut shared, path, profile, options, pending);
                provenance.record(
                    &SqlSourceKey::profile(SqlProjection::Embedded, options),
                    pending.span,
                    std::mem::take(&mut shared.failures),
                );
                if let Some(call) = call {
                    let lines = pending.sites.iter().map(|(_, site)| site.line).collect();
                    completed.push((path.clone(), profile, pending.span, call, lines));
                }
            }
        }
        if !provenance.is_empty() {
            prerequisites.push((path.clone(), provenance));
        }
    }
    (completed, prerequisites, locks)
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
