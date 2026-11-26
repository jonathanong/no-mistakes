use super::File;
use crate::codebase::dependencies::extract::ImportKind;
use crate::codebase::postgres::query_annotation::module_ids::{ModuleId, ModuleIds};
use crate::fx::{FxHashMap, FxHashSet};
use std::path::{Path, PathBuf};

pub(super) struct CandidateGraph {
    pub(super) order: Vec<ModuleId>,
    pub(super) direct_deps: FxHashMap<ModuleId, Vec<ModuleId>>,
    pub(super) cyclic: FxHashSet<ModuleId>,
    pub(super) root_reachable: Vec<FxHashSet<ModuleId>>,
}

/// Discover modules statically reachable from at least two roots in O(|V|+|E|)
/// using a 2-capped reach counter, then order them in deterministic post-order.
pub(super) fn discover(
    files: &FxHashMap<PathBuf, File<'_>>,
    roots: &[PathBuf],
    ids: &ModuleIds,
    resolve: &impl Fn(&str, &Path) -> Option<PathBuf>,
) -> CandidateGraph {
    let mut direct_deps: FxHashMap<ModuleId, Vec<ModuleId>> = FxHashMap::default();
    let mut reach_count: FxHashMap<ModuleId, u8> = FxHashMap::default();
    let mut root_reachable: Vec<FxHashSet<ModuleId>> = Vec::with_capacity(roots.len());
    let mut stack = Vec::new();

    for root in roots {
        let mut seen_in_root = FxHashSet::default();
        let root_id = ids.id(root);
        seen_in_root.insert(root_id);
        stack.push(root_id);
        while let Some(id) = stack.pop() {
            let count = reach_count.entry(id).or_insert(0);
            *count = count.saturating_add(1).min(2);
            let deps = direct_deps
                .entry(id)
                .or_insert_with(|| static_deps(files, ids, id, resolve));
            for &dep in deps.iter() {
                if seen_in_root.insert(dep) {
                    stack.push(dep);
                }
            }
        }
        root_reachable.push(seen_in_root);
    }

    let mut shared_ids = reach_count
        .into_iter()
        .filter_map(|(id, count)| (count >= 2).then_some(id))
        .collect::<Vec<_>>();
    shared_ids.sort_by(|left, right| ids.path(*left).cmp(ids.path(*right)));

    let mut state: FxHashMap<ModuleId, u8> = FxHashMap::default();
    let mut order = Vec::with_capacity(shared_ids.len());
    let mut cyclic = FxHashSet::default();
    for id in shared_ids {
        post_order(id, &direct_deps, &mut state, &mut order, &mut cyclic);
    }

    CandidateGraph {
        order,
        direct_deps,
        cyclic,
        root_reachable,
    }
}

fn static_deps(
    files: &FxHashMap<PathBuf, File<'_>>,
    ids: &ModuleIds,
    id: ModuleId,
    resolve: &impl Fn(&str, &Path) -> Option<PathBuf>,
) -> Vec<ModuleId> {
    let path = ids.path(id);
    let Some(file) = files.get(path) else {
        return Vec::new();
    };
    let mut deps = if file.facts.precise_records {
        file.module
            .ts
            .imports
            .iter()
            .filter(|import| import.kind == ImportKind::Static && import.function_scope.is_none())
            .filter_map(|import| file.module.target(&import.specifier, path, resolve))
            .filter(|target| files.contains_key(*target))
            .map(|target| ids.id(target))
            .collect::<Vec<_>>()
    } else {
        referenced_deps(file, path, files, ids, resolve)
    };
    deps.sort_by(|left, right| ids.path(*left).cmp(ids.path(*right)));
    deps.dedup();
    deps
}

fn referenced_deps(
    file: &File<'_>,
    path: &Path,
    files: &FxHashMap<PathBuf, File<'_>>,
    ids: &ModuleIds,
    resolve: &impl Fn(&str, &Path) -> Option<PathBuf>,
) -> Vec<ModuleId> {
    use super::super::super::evaluate::run::preload::{collect_expr_names, collect_steps_names};
    let imports = &file.module.imports;
    let mut names = FxHashSet::default();
    if !imports.is_empty() {
        collect_steps_names(&file.facts.roots, imports, &mut names);
        for call in &file.facts.unmodeled_calls {
            collect_expr_names(call, imports, &mut names);
        }
        for expr in file.facts.globals.values() {
            collect_expr_names(expr, imports, &mut names);
        }
    }
    names
        .into_iter()
        .filter_map(|name| {
            let index = imports.get(name)?;
            let specifier = &file.module.ts.imported_bindings[*index].specifier;
            file.module.target(specifier, path, resolve)
        })
        .filter(|target| files.contains_key(*target))
        .map(|target| ids.id(target))
        .collect()
}

fn post_order(
    root: ModuleId,
    direct_deps: &FxHashMap<ModuleId, Vec<ModuleId>>,
    state: &mut FxHashMap<ModuleId, u8>,
    order: &mut Vec<ModuleId>,
    cyclic: &mut FxHashSet<ModuleId>,
) {
    if state.get(&root).copied().unwrap_or(0) != 0 {
        return;
    }
    state.insert(root, 1);
    let mut stack = vec![(root, 0usize)];
    while let Some((id, next_idx)) = stack.last_mut() {
        let id = *id;
        let dep = direct_deps
            .get(&id)
            .and_then(|deps| deps.get(*next_idx))
            .copied();
        if let Some(dep) = dep {
            *next_idx += 1;
            match state.get(&dep).copied().unwrap_or(0) {
                0 => {
                    state.insert(dep, 1);
                    stack.push((dep, 0));
                }
                1 => {
                    cyclic.insert(dep);
                    cyclic.insert(id);
                }
                _ => {
                    if cyclic.contains(&dep) {
                        cyclic.insert(id);
                    }
                }
            }
        } else {
            stack.pop();
            state.insert(id, 2);
            order.push(id);
            if cyclic.contains(&id) {
                if let Some(&(parent, _)) = stack.last() {
                    cyclic.insert(parent);
                }
            }
        }
    }
}
