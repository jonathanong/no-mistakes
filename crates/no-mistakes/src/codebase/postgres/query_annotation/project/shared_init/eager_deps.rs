use super::File;
use crate::codebase::postgres::query_annotation::evaluate::run::preload::collect_top_level_steps_names;
use crate::codebase::postgres::query_annotation::module_ids::{ModuleId, ModuleIds};
use crate::codebase::postgres::query_annotation::Expr;
use crate::fx::{FxHashMap, FxHashSet};
use std::path::{Path, PathBuf};

pub(super) fn for_module(
    files: &FxHashMap<PathBuf, File<'_>>,
    ids: &ModuleIds,
    id: ModuleId,
    direct: &[ModuleId],
    resolve: &impl Fn(&str, &Path) -> Option<PathBuf>,
) -> Vec<ModuleId> {
    let path = ids.path(id);
    let file = &files[path];
    if file.facts.precise_records {
        return direct.to_vec();
    }
    if file.module.imports.is_empty() {
        return Vec::new();
    }
    let mut names = FxHashSet::default();
    collect_top_level_steps_names(&file.facts.roots, &file.module.imports, &mut names);
    let mut deps = Vec::new();
    for name in names {
        let binding = &file.module.ts.imported_bindings[file.module.imports[name]];
        if let Some(target) = file.module.target(&binding.specifier, path, resolve) {
            let mut visiting = FxHashSet::default();
            collect_export_targets(
                files,
                ids,
                target,
                &binding.imported,
                resolve,
                &mut visiting,
                &mut deps,
            );
        }
    }
    deps.sort_by(|left, right| ids.path(*left).cmp(ids.path(*right)));
    deps.dedup();
    deps
}

fn collect_export_targets<'a>(
    files: &'a FxHashMap<PathBuf, File<'_>>,
    ids: &ModuleIds,
    path: &'a Path,
    name: &'a str,
    resolve: &impl Fn(&str, &Path) -> Option<PathBuf>,
    visiting: &mut FxHashSet<(&'a Path, &'a str)>,
    deps: &mut Vec<ModuleId>,
) {
    if !visiting.insert((path, name)) {
        return;
    }
    let Some(file) = files.get(path) else {
        return;
    };
    if let Some(&index) = file.module.exports.get(name) {
        let binding = &file.module.ts.exported_bindings[index];
        let import = (!file.facts.globals.contains_key(&binding.local))
            .then(|| file.module.imports.get(binding.local.as_str()))
            .flatten()
            .map(|&idx| &file.module.ts.imported_bindings[idx]);
        let forwarded = binding
            .specifier
            .as_deref()
            .map(|specifier| (specifier, binding.local.as_str()))
            .or_else(|| import.map(|imp| (imp.specifier.as_str(), imp.imported.as_str())));
        if let Some((specifier, next_name)) = forwarded {
            if let Some(target) = file.module.target(specifier, path, resolve) {
                collect_export_targets(files, ids, target, next_name, resolve, visiting, deps);
            }
        } else if file
            .facts
            .globals
            .get(&binding.local)
            .is_some_and(|expr| !matches!(expr, Expr::Unknown))
        {
            deps.push(ids.id(path));
        }
        return;
    }
    if name == "default" {
        return;
    }
    for specifier in &file.module.ts.star_reexport_specifiers {
        if let Some(target) = file.module.target(specifier, path, resolve) {
            collect_export_targets(files, ids, target, name, resolve, visiting, deps);
        }
    }
}
