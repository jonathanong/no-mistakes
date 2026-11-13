//! Exactly caller -> async factory -> direct prepared append-effect exports.
use crate::codebase::check_facts::{
    sql_source_prerequisites::{source_failed, FailedSqlSources},
    CheckFileFacts,
};
use crate::codebase::ts_source::FileIdMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub(in super::super) fn project(
    files: &mut FileIdMap<CheckFileFacts>,
    resolve: &impl Fn(&str, &Path) -> Option<PathBuf>,
) -> Vec<(PathBuf, super::Span, FailedSqlSources)> {
    let mut completed = Vec::new();
    let mut effect_resolver = super::super::effects::Resolver::new(files, resolve);
    let mut failed = Vec::new();
    for (path, file) in files.iter() {
        let Some(heads) = file.query_annotation_heads.as_deref() else {
            continue;
        };
        for pending in &heads.pending_async_returns {
            let Some(target) = resolve(&pending.specifier, path) else {
                continue;
            };
            if source_failed(files, &target) {
                failed.push((path.clone(), pending.span, [target].into_iter().collect()));
                continue;
            }
            let Some(summary) = files
                .get(&target)
                .and_then(|file| file.query_annotation_heads.as_deref())
                .and_then(|heads| heads.exported_async_returns.get(&pending.imported))
            else {
                continue;
            };
            let mut effect_failures = FailedSqlSources::new();
            let proven = summary.required.iter().all(|effect| {
                let (proven, failed) = effect_resolver.project(&target, effect);
                effect_failures.extend(failed);
                proven
            });
            if !effect_failures.is_empty() {
                failed.push((path.clone(), pending.span, effect_failures));
            }
            if proven {
                completed.push((path.clone(), pending.span, summary.proof));
            }
        }
    }
    drop(effect_resolver);
    for (path, span, proof) in completed {
        let Some(heads) = files
            .get_mut(&path)
            .and_then(|file| file.query_annotation_heads.as_mut())
        else {
            continue;
        };
        if let Some(call) = Arc::make_mut(heads).calls.get_mut(&span) {
            *call = proof;
        }
    }
    failed
}
