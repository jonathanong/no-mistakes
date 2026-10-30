//! Drop closed-name proofs when any import in the file is unresolved.
use super::{AnnotationHeads, HeadProof};
use crate::codebase::check_facts::CheckFileFacts;
use crate::codebase::dependencies::extract::ExtractedImport;
use crate::codebase::ts_source::FileIdMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub(crate) fn retain(
    files: &mut FileIdMap<CheckFileFacts>,
    resolve: &impl Fn(&str, &Path) -> Option<PathBuf>,
) {
    let blocked: Vec<PathBuf> = files
        .iter()
        .filter(|(path, file)| {
            file.query_annotation_heads
                .as_deref()
                .is_some_and(|heads| !heads.import_gated_finite.is_empty())
                && unresolved(&file.ts.imports, path, resolve)
        })
        .map(|(path, _)| path.clone())
        .collect();
    for path in blocked {
        let Some(heads) = files
            .get_mut(&path)
            .and_then(|file| file.query_annotation_heads.as_mut())
        else {
            continue;
        };
        let heads: &mut AnnotationHeads = Arc::make_mut(heads);
        for span in heads.import_gated_finite.clone() {
            if matches!(heads.calls.get(&span), Some(HeadProof::Literal)) {
                heads.calls.insert(span, HeadProof::Unproven);
            }
        }
    }
}

fn unresolved(
    imports: &[ExtractedImport],
    from: &Path,
    resolve: &impl Fn(&str, &Path) -> Option<PathBuf>,
) -> bool {
    imports
        .iter()
        .any(|import| import.computed || resolve(&import.specifier, from).is_none())
}
