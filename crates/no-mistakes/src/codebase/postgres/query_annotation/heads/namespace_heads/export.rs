//! Bounded exact runtime export walk over request-prepared file facts.
use super::Pending;
use crate::codebase::{
    check_facts::CheckFileFacts,
    postgres::query_annotation::heads::HeadProof,
    ts_source::{
        module_facts::{ModuleBinding, TypeScriptModuleFacts},
        FileIdMap,
    },
};
use std::path::{Path, PathBuf};

pub(in crate::codebase::postgres::query_annotation::heads) fn project(
    files: &FileIdMap<CheckFileFacts>,
    resolve: &dyn Fn(&str, &Path) -> Option<PathBuf>,
    from: &Path,
    pending: &Pending,
) -> Option<HeadProof> {
    let target = resolve(&pending.specifier, from)?;
    export(files, resolve, &target, &pending.exported, &mut Vec::new())
}

fn export(
    files: &FileIdMap<CheckFileFacts>,
    resolve: &dyn Fn(&str, &Path) -> Option<PathBuf>,
    file: &Path,
    name: &str,
    visited: &mut Vec<(PathBuf, String)>,
) -> Option<HeadProof> {
    let step = (file.to_path_buf(), name.to_string());
    if visited.len() >= 8 || visited.contains(&step) {
        return None;
    }
    visited.push(step);
    let module = module(files, file)?;
    let mut exports = module.exports.iter().filter(|value| value.exported == name);
    let value = exports.next()?;
    if exports.next().is_some() || value.type_only {
        return None;
    }
    if let Some(specifier) = &value.specifier {
        let target = resolve(specifier, file)?;
        return export(files, resolve, &target, &value.local, visited);
    }
    let mut bindings = module.bindings.iter().filter(|binding| {
        binding.name == value.local
            && binding.runtime
            && module
                .scopes
                .iter()
                .any(|scope| scope.id == binding.scope_id && scope.parent_id.is_none())
    });
    let binding = bindings.next()?;
    if bindings.next().is_some() {
        return None;
    }
    if binding.imported {
        return imported(files, resolve, file, module, binding, visited);
    }
    files
        .get(file)?
        .query_annotation_heads
        .as_deref()?
        .exported_helpers
        .get(name)
        .copied()
}

fn imported(
    files: &FileIdMap<CheckFileFacts>,
    resolve: &dyn Fn(&str, &Path) -> Option<PathBuf>,
    file: &Path,
    module: &TypeScriptModuleFacts,
    binding: &ModuleBinding,
    visited: &mut Vec<(PathBuf, String)>,
) -> Option<HeadProof> {
    let mut imports = module
        .imports
        .iter()
        .filter(|import| !import.type_only)
        .flat_map(|import| import.bindings.iter().map(move |entry| (import, entry)))
        .filter(|(_, entry)| {
            entry.binding_id == binding.id
                && !entry.type_only
                && entry.kind != "namespace"
                && entry.imported != "*"
        });
    let (import, entry) = imports.next()?;
    if imports.next().is_some() {
        return None;
    }
    let target = resolve(&import.specifier, file)?;
    export(files, resolve, &target, &entry.imported, visited)
}

fn module<'a>(
    files: &'a FileIdMap<CheckFileFacts>,
    file: &Path,
) -> Option<&'a TypeScriptModuleFacts> {
    let facts = files.get(file)?;
    if facts.parse_error.is_some()
        || facts.ts.parse_error.is_some()
        || facts.ts.operational_error.is_some()
        || facts.ts.fatal_parse_error
    {
        return None;
    }
    let module = facts.ts.module_bindings.as_deref()?;
    module.diagnostics.is_empty().then_some(module)
}
