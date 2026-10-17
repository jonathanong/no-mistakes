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
                // Dynamic/computed imports are never grounds for pruning,
                // even when their currently visible literal target resolves.
                if import.computed || import.kind == ImportKind::Dynamic {
                    visits.get_mut(path).unwrap().demanded = true;
                }
                if import.computed {
                    continue;
                }
                let target = module
                    .target(&import.specifier, path, resolve)
                    .and_then(|target| files.get_key_value(target))
                    .map(|(path, _)| path.as_path());
                let Some(target) = target else {
                    visits.get_mut(path).unwrap().demanded = true;
                    continue;
                };
                if let Some(child) = visits.get(target) {
                    let low = child.active.then_some(child.index);
                    let demanded = child.demanded;
                    let parent = visits.get_mut(path).unwrap();
                    if let Some(low) = low {
                        parent.low = parent.low.min(low);
                    }
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
            demanded: !files[path].executors.is_empty(),
        },
    );
    component.push(path);
    frames.push(Frame {
        path,
        next_import: 0,
    });
}
