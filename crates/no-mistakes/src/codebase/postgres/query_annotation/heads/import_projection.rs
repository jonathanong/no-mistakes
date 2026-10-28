//! Resolve only direct source imports against the request's prepared file map.
use super::{AnnotationHeads, HeadProof, Span};
use crate::codebase::check_facts::CheckFileFacts;
use crate::codebase::ts_source::FileIdMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub(crate) fn project(
    files: &mut FileIdMap<CheckFileFacts>,
    resolve: impl Fn(&str, &Path) -> Option<PathBuf>,
) {
    let mut completed: Vec<(PathBuf, Span, HeadProof)> = Vec::new();
    for (path, file) in files.iter() {
        let Some(heads) = file.query_annotation_heads.as_deref() else {
            continue;
        };
        for pending in &heads.pending_helpers {
            let Some(target) = resolve(&pending.specifier, path) else {
                continue;
            };
            let Some(proof) = files
                .get(&target)
                .and_then(|target| target.query_annotation_heads.as_deref())
                .and_then(|heads| heads.exported_helpers.get(&pending.imported))
                .copied()
            else {
                continue;
            };
            completed.push((path.clone(), pending.span, proof));
        }
    }
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
}
