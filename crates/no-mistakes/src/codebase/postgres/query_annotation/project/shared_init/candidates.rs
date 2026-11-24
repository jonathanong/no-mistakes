use super::File;
use crate::codebase::dependencies::extract::ImportKind;
use crate::codebase::postgres::query_annotation::module_ids::{ModuleId, ModuleIds};
use crate::fx::{FxHashMap, FxHashSet};
use std::path::{Path, PathBuf};

pub(super) struct CandidateGraph {
    pub(super) order: Vec<ModuleId>,
    pub(super) direct_deps: FxHashMap<ModuleId, Vec<ModuleId>>,
    pub(super) cyclic: FxHashSet<ModuleId>,
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
    let mut seen_in_root = FxHashSet::default();
    let mut stack = Vec::new();

    for root in roots {
        seen_in_root.clear();
        let root_id = ids.id(root);
        seen_in_root.insert(root_id);
        stack.push(root_id);
        while let Some(id) = stack.pop() {
            let count = reach_count.entry(id).or_insert(0);
            if *count >= 2 {
                continue;
            }
            *count += 1;
            let deps = direct_deps
                .entry(id)
                .or_insert_with(|| static_deps(files, ids, id, resolve));
            for &dep in deps.iter() {
                if seen_in_root.insert(dep) && reach_count.get(&dep).copied().unwrap_or(0) < 2 {
                    stack.push(dep);
                }
            }
        }
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
    id: ModuleId,
    direct_deps: &FxHashMap<ModuleId, Vec<ModuleId>>,
    state: &mut FxHashMap<ModuleId, u8>,
    order: &mut Vec<ModuleId>,
    cyclic: &mut FxHashSet<ModuleId>,
) {
    match state.get(&id).copied().unwrap_or(0) {
        2 => return,
        1 => {
            cyclic.insert(id);
            return;
        }
        _ => {}
    }
    state.insert(id, 1);
    if let Some(deps) = direct_deps.get(&id) {
        for &dep in deps {
            post_order(dep, direct_deps, state, order, cyclic);
            if cyclic.contains(&dep) {
                cyclic.insert(id);
            }
        }
    }
    state.insert(id, 2);
    order.push(id);
}
