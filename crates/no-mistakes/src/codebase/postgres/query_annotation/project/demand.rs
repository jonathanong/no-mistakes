use super::{File, Module};
use crate::codebase::dependencies::extract::ImportKind;
use crate::fx::{fx_map, FxHashMap};
use std::{
    borrow::Borrow,
    hash::Hash,
    path::{Path, PathBuf},
};

struct Visit {
    index: usize,
    low: usize,
    active: bool,
    demanded: bool,
}

struct Frame<'a> {
    path: &'a Path,
    next_import: usize,
}

/// Project reachability directly from the prepared module catalog. Annotation
/// projection runs before DepGraph construction and also serves supplied-file
/// extraction; rebuilding a codebase graph here would change resolver semantics.
/// Only scoped executor spans can produce annotation findings. Excluded callers
/// that reach them retain their contextual argument flow. Unresolved external loads
/// invalidate values but cannot enter another catalog file, so they do not seed
/// demand. All resolved runtime imports and re-exports remain reachability edges.
/// Soundness invariant: findings occur only at scoped executor spans, and the
/// evaluator reaches another file only through these runtime import/re-export
/// edges. An SCC with no path to a scoped executor cannot affect a finding.
/// New non-import file reach must be mirrored here before pruning stays sound.
/// Iterative Tarjan visits SCCs bottom-up without a second adjacency index.
fn roots_for_scope<'a, K: Borrow<Path> + Eq + Hash, F>(
    files: &'a FxHashMap<K, F>,
    resolve: &impl Fn(&str, &Path) -> Option<PathBuf>,
    module: impl Fn(&F) -> &Module,
    is_seed: impl Fn(&F) -> bool,
) -> Vec<&'a Path> {
    if !files.values().any(&is_seed) {
        return Vec::new();
    }
    let mut visits: FxHashMap<&Path, Visit> = fx_map();
    let mut component = Vec::new();
    let mut frames = Vec::<Frame<'_>>::new();
    for path in files.keys().map(|key| -> &Path { key.borrow() }) {
        if visits.contains_key(path) {
            continue;
        }
        enter(
            path,
            is_seed(&files[path]),
            &mut visits,
            &mut component,
            &mut frames,
        );
        while let Some(frame) = frames.last_mut() {
            let path = frame.path;
            let module = module(&files[path]);
            // A positive answer cannot change: classify the remaining targets
            // independently later instead of resolving unused imports here.
            let import = if visits[path].demanded {
                None
            } else {
                module.ts.imports.get(frame.next_import)
            };
            if let Some(import) = import {
                frame.next_import += 1;
                if import.kind == ImportKind::Type {
                    continue;
                }
                // Computed loads have no evaluator module target. Their operand
                // expressions still contribute any nested literal import edges.
                if import.computed {
                    continue;
                }
                let Some(target) = module.target(&import.specifier, path, resolve) else {
                    continue;
                };
                // A successfully resolved edge outside the prepared catalog
                // cannot lead back to a catalog executor. Parse-failed inventory
                // files are omitted from that catalog and are the same dead end
                // only while a parse failure yields no query facts. If recovery
                // later emits partial facts, this edge must stay conservative.
                let Some((target, _)) = files.get_key_value(target) else {
                    continue;
                };
                let target: &Path = target.borrow();
                if let Some(child) = visits.get(target) {
                    let low = child.active.then_some(child.index);
                    let demanded = child.demanded;
                    let parent = visits.get_mut(path).unwrap();
                    if let Some(low) = low {
                        parent.low = parent.low.min(low);
                    }
                    parent.demanded |= demanded;
                } else {
                    enter(
                        target,
                        is_seed(&files[target]),
                        &mut visits,
                        &mut component,
                        &mut frames,
                    );
                }
                continue;
            }
            let low = visits[path].low;
            if visits[path].index == low {
                let start = component
                    .iter()
                    .rposition(|member| *member == path)
                    .unwrap();
                let demanded = component[start..]
                    .iter()
                    .any(|member| visits[member].demanded);
                for member in component.drain(start..) {
                    let visit = visits.get_mut(member).unwrap();
                    visit.active = false;
                    visit.demanded = demanded;
                }
            }
            frames.pop();
            if let Some(parent) = frames.last() {
                let demanded = visits[path].demanded;
                let parent = visits.get_mut(parent.path).unwrap();
                parent.low = parent.low.min(low);
                parent.demanded |= demanded;
            }
        }
    }
    visits
        .into_iter()
        .filter_map(|(path, visit)| visit.demanded.then_some(path))
        .collect()
}

fn enter<'a>(
    path: &'a Path,
    demanded: bool,
    visits: &mut FxHashMap<&'a Path, Visit>,
    component: &mut Vec<&'a Path>,
    frames: &mut Vec<Frame<'a>>,
) {
    let index = visits.len();
    visits.insert(
        path,
        Visit {
            index,
            low: index,
            active: true,
            demanded,
        },
    );
    component.push(path);
    frames.push(Frame {
        path,
        next_import: 0,
    });
}

pub(super) fn roots<'a>(
    files: &'a FxHashMap<PathBuf, File<'_>>,
    resolve: &impl Fn(&str, &Path) -> Option<PathBuf>,
) -> Vec<&'a Path> {
    roots_for_scope(
        files,
        resolve,
        |file| &file.module,
        |file| file.facts.in_scope != Some(false) && !file.executors.is_empty(),
    )
}

pub(super) fn roots_before_scoping<'a>(
    files: &'a FxHashMap<PathBuf, File<'_>>,
    resolve: &impl Fn(&str, &Path) -> Option<PathBuf>,
) -> Vec<&'a Path> {
    roots_for_scope(
        files,
        resolve,
        |file| &file.module,
        |file| !file.executors.is_empty(),
    )
}

pub(super) type SyntaxModules<'a> = FxHashMap<&'a Path, (&'a Module, bool)>;

pub(super) fn potential_roots<'a>(
    files: &SyntaxModules<'a>,
    resolve: &impl Fn(&str, &Path) -> Option<PathBuf>,
) -> Vec<&'a Path> {
    // The selection borrows the catalog path, never the temporary hash table.
    roots_for_scope(files, resolve, |entry| entry.0, |entry| entry.1)
        .into_iter()
        .map(|path| *files.get_key_value(path).unwrap().0)
        .collect()
}

mod forward;
pub(super) use forward::reachable;
