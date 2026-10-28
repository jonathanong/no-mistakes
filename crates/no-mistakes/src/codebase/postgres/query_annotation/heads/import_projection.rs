//! Resolve only direct source imports against the request's prepared file map.
use super::{AnnotationHeads, HeadProof, Span};
use crate::codebase::check_facts::CheckFileFacts;
use crate::codebase::ts_source::FileIdMap;
use crate::fx::{FxHashMap, FxHashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub(crate) fn project(
    files: &mut FileIdMap<CheckFileFacts>,
    resolve: impl Fn(&str, &Path) -> Option<PathBuf>,
) {
    let mut completed: Vec<(PathBuf, Span, HeadProof)> = Vec::new();
    let mut effects: FxHashMap<PathBuf, FxHashSet<Span>> = FxHashMap::default();
    let mut callbacks = Vec::new();
    for (path, file) in files.iter() {
        let Some(heads) = file.query_annotation_heads.as_deref() else {
            continue;
        };
        for pending in &heads.pending_imports {
            let Some(target) = resolve(&pending.specifier, path) else {
                continue;
            };
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
            let Some(target) = resolve(&pending.specifier, path) else {
                continue;
            };
            let proven = files
                .get(&target)
                .and_then(|target| target.query_annotation_heads.as_deref())
                .is_some_and(|heads| heads.exported_returned_params.contains(&pending.imported));
            if proven {
                completed.push((path.clone(), pending.span, pending.proof));
            }
        }
        for pending in &heads.pending_namespaces {
            if let Some(proof) = super::namespace_heads::project(files, &resolve, path, pending) {
                completed.push((path.clone(), pending.span, proof));
            }
        }
        for pending in &heads.pending_callbacks {
            let Some(target) = resolve(&pending.specifier, path) else {
                continue;
            };
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
            let Some(target) = resolve(&pending.specifier, path) else {
                continue;
            };
            let proven = files
                .get(&target)
                .and_then(|target| target.query_annotation_heads.as_deref())
                .and_then(|heads| heads.exported_effects.get(&pending.imported))
                .is_some_and(|signature| signature.accepts(pending.shape));
            if proven {
                effects
                    .entry(path.clone())
                    .or_default()
                    .insert(pending.span);
            }
        }
    }
    super::async_returns::project(files, &resolve);
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
            if condition
                .required
                .iter()
                .all(|span| approved.contains(span))
            {
                heads.calls.insert(condition.span, HeadProof::Literal);
            }
        }
    }
}
