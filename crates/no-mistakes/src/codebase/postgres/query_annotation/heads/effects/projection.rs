//! Direct prepared export lookups memoize both approval and rejection.
use super::super::Span;
use crate::codebase::check_facts::CheckFileFacts;
use crate::codebase::ts_source::FileIdMap;
use crate::fx::{FxHashMap, FxHashSet};
use std::path::{Path, PathBuf};

pub(in super::super) fn approved(
    files: &FileIdMap<CheckFileFacts>,
    resolve: &impl Fn(&str, &Path) -> Option<PathBuf>,
) -> FxHashMap<PathBuf, FxHashSet<Span>> {
    let mut result: FxHashMap<PathBuf, FxHashSet<Span>> = FxHashMap::default();
    let mut exports = FxHashMap::default();
    for (path, file) in files.iter() {
        let Some(heads) = file.query_annotation_heads.as_deref() else {
            continue;
        };
        for pending in &heads.pending_effects {
            // Shape stays at the call site; only the immutable target lookup is shared.
            let target = exports
                .entry((path, pending.specifier.as_str(), pending.imported.as_str()))
                .or_insert_with(|| {
                    if let Some(observer) = crate::diagnostics::current() {
                        observer.increment("postgres.annotation.effect_export_lookups", 1);
                    }
                    resolve(&pending.specifier, path)
                        .and_then(|target| files.get(&target))
                        .and_then(|file| file.query_annotation_heads.as_deref())
                        .and_then(|heads| {
                            heads
                                .exported_effects
                                .get(&pending.imported)
                                .map(|signature| {
                                    (
                                        *signature,
                                        heads.exported_closed_effects.contains(&pending.imported),
                                    )
                                })
                        })
                });
            if target.is_some_and(|(signature, closed)| {
                (!pending.require_plain || closed) && pending.accepts(signature)
            }) {
                result.entry(path.clone()).or_default().insert(pending.span);
            }
        }
    }
    result
}
