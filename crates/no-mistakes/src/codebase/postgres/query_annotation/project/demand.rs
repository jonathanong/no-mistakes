use super::File;
use crate::codebase::dependencies::extract::ImportKind;
use crate::fx::{fx_map, FxHashMap};
use std::path::{Path, PathBuf};

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
/// Only prepared executor spans can emit events. Unresolved external loads
/// invalidate values but cannot enter another catalog file, so they do not seed
/// demand. All resolved runtime imports and re-exports remain reachability edges.
/// Soundness invariant: events occur only at prepared executor spans, and the
/// evaluator reaches another file only through these runtime import/re-export
/// edges. An executor-free SCC with no path to an executor cannot emit an event.
/// New non-import file reach must be mirrored here before pruning stays sound.
/// Iterative Tarjan traversal needs no competing forward/reverse adjacency and
/// keeps long import chains off the native stack. Completed SCCs are bottom-up.
pub(super) fn roots<'a>(
    files: &'a FxHashMap<PathBuf, File<'_>>,
    resolve: &impl Fn(&str, &Path) -> Option<PathBuf>,
) -> Vec<&'a Path> {
    let mut visits: FxHashMap<&Path, Visit> = fx_map();
    let mut component = Vec::new();
    let mut frames = Vec::<Frame<'_>>::new();
    for path in files.keys() {
        if visits.contains_key(path.as_path()) {
            continue;
        }
        enter(path, files, &mut visits, &mut component, &mut frames);
        while let Some(frame) = frames.last_mut() {
            let path = frame.path;
            let module = &files[path].module;
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
                let target = target.as_path();
                if let Some(child) = visits.get(target) {
                    let low = child.active.then_some(child.index);
                    let demanded = child.demanded;
                    let parent = visits.get_mut(path).unwrap();
                    if let Some(low) = low {
                        parent.low = parent.low.min(low);
                    }
                    // Excluded files stay on the import path. Root selection
                    // applies the path filter after the whole graph is demanded.
                    parent.demanded |= demanded;
                } else {
                    enter(target, files, &mut visits, &mut component, &mut frames);
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
                let parent_path = parent.path;
                let parent = visits.get_mut(parent_path).unwrap();
                parent.low = parent.low.min(low);
                parent.demanded |= demanded;
            }
        }
    }
    visits
        .into_iter()
        .filter_map(|(path, visit)| {
            let file = &files[path];
            // Executor-free importers still supply SQL to an included call.
            // An excluded file that contains its own executor call does not.
            (visit.demanded && (file.facts.annotation_event || file.executors.is_empty()))
                .then_some(path)
        })
        .collect()
}

fn enter<'a>(
    path: &'a Path,
    files: &FxHashMap<PathBuf, File<'_>>,
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
            // Only an included executor call seeds the walk. Excluded tests that
            // call the executor themselves are not roots. Callers that only
            // forward SQL stay reachable through the import edges below.
            demanded: !files[path].executors.is_empty() && files[path].facts.annotation_event,
        },
    );
    component.push(path);
    frames.push(Frame {
        path,
        next_import: 0,
    });
}
