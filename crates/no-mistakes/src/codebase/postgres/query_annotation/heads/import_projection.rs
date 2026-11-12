//! Resolve only direct source imports against the request's prepared file map.
use super::{AnnotationHeads, HeadProof, Span};
use crate::codebase::check_facts::{
    sql_source_prerequisites::{source_failed, FailedSqlSources},
    CheckFileFacts,
};
use crate::codebase::ts_source::FileIdMap;
use crate::fx::{FxHashMap, FxHashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
mod record_failures;

pub(crate) fn project(
    files: &mut FileIdMap<CheckFileFacts>,
    resolve: impl Fn(&str, &Path) -> Option<PathBuf>,
) {
    let observer = crate::diagnostics::current();
    let mut effect_resolver = super::effects::Resolver::new(files, &resolve);
    let mut completed: Vec<(PathBuf, Span, HeadProof)> = Vec::new();
    let mut effects: FxHashMap<PathBuf, FxHashSet<Span>> = FxHashMap::default();
    let mut callbacks = Vec::new();
    let mut failed_sources = Vec::<(PathBuf, Span, FailedSqlSources)>::new();
    for (path, file) in files.iter() {
        let Some(heads) = file.query_annotation_heads.as_deref() else {
            continue;
        };
        for pending in &heads.pending_imports {
            let Some(target) = resolve(&pending.specifier, path) else {
                continue;
            };
            if source_failed(files, &target) {
                failed_sources.push((path.clone(), pending.span, [target].into_iter().collect()));
                continue;
            }
            let Some(proof) = files
                .get(&target)
                .and_then(|target| target.query_annotation_heads.as_deref())
                .and_then(|heads| match pending.kind {
                    super::helper_heads::Kind::Helper => {
                        heads.exported_helpers.get(&pending.imported).copied()
                    }
                    super::helper_heads::Kind::Value => {
                        heads.exported_values.get(&pending.imported).copied()
                    }
                    super::helper_heads::Kind::Record => heads
                        .exported_records
                        .get(&pending.imported)
                        .map(|summary| summary.proof()),
                    super::helper_heads::Kind::PlainRecord => heads
                        .exported_records
                        .get(&pending.imported)
                        .and_then(|summary| match summary {
                            super::helper_records::Summary::Plain(proof) => Some(*proof),
                            _ => None,
                        }),
                    super::helper_heads::Kind::Builder => heads
                        .exported_records
                        .get(&pending.imported)
                        .and_then(|summary| match summary {
                            super::helper_records::Summary::Builder { proof, .. } => Some(*proof),
                            _ => None,
                        }),
                })
            else {
                continue;
            };
            completed.push((path.clone(), pending.span, proof));
        }
        for pending in &heads.pending_returned_params {
            let (proven, failed) = super::returned_params::project(files, &resolve, path, pending);
            if !failed.is_empty() {
                failed_sources.push((path.clone(), pending.span, failed));
            }
            if proven {
                completed.push((path.clone(), pending.span, pending.proof));
            }
        }
        for pending in &heads.pending_namespaces {
            let (proof, failed) = super::namespace_heads::project(files, &resolve, path, pending);
            if !failed.is_empty() {
                failed_sources.push((path.clone(), pending.span, failed));
            }
            if let Some(proof) = proof {
                completed.push((path.clone(), pending.span, proof));
            }
        }
        for pending in &heads.pending_callbacks {
            let Some(target) = resolve(&pending.specifier, path) else {
                continue;
            };
            if source_failed(files, &target) {
                for span in pending.affected_spans() {
                    failed_sources.push((
                        path.clone(),
                        span,
                        [target.clone()].into_iter().collect(),
                    ));
                }
                continue;
            }
            let Some(summary) = files
                .get(&target)
                .and_then(|target| target.query_annotation_heads.as_deref())
                .and_then(|heads| heads.exported_callbacks.get(&pending.imported))
            else {
                continue;
            };
            if let Some(span) = pending.proven_executor(summary) {
                callbacks.push((path.clone(), span));
            }
        }
        for pending in &heads.pending_effects {
            let (proven, failed) = effect_resolver.project(path, pending);
            if !failed.is_empty() {
                failed_sources.push((path.clone(), pending.span, failed));
            }
            if proven {
                effects
                    .entry(path.clone())
                    .or_default()
                    .insert(pending.span);
            }
        }
    }
    drop(effect_resolver);
    failed_sources.extend(super::async_returns::project(files, &resolve));
    for (path, span, proof) in completed {
        let Some(heads) = files
            .get_mut(&path)
            .and_then(|file| file.query_annotation_heads.as_mut())
        else {
            continue;
        };
        let heads: &mut AnnotationHeads = Arc::make_mut(heads);
        if let Some(call) = heads.calls.get_mut(&span) {
            *call = proof;
        }
    }
    for (path, span) in callbacks {
        let Some(heads) = files
            .get_mut(&path)
            .and_then(|file| file.query_annotation_heads.as_mut())
        else {
            continue;
        };
        let heads = Arc::make_mut(heads);
        if let Some(proof) = heads.calls.get_mut(&span) {
            *proof = HeadProof::Literal;
        }
    }
    for (path, approved) in effects {
        let Some(heads) = files
            .get_mut(&path)
            .and_then(|file| file.query_annotation_heads.as_mut())
        else {
            continue;
        };
        let heads: &mut AnnotationHeads = Arc::make_mut(heads);
        for condition in &heads.conditional_effects {
            if condition.required.iter().all(|span| {
                if let Some(observer) = &observer {
                    observer.increment("postgres.annotation.conditional_effect_approval_checks", 1);
                }
                approved.contains(span)
            }) {
                for &span in &condition.spans {
                    heads.calls.insert(span, HeadProof::Literal);
                }
            }
        }
    }
    for (path, span, failed) in failed_sources {
        record_failures::record(files, &path, span, failed);
    }
    super::import_gate::retain(files, &resolve);
}
