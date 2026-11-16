//! Where a registry binding is defined.
use crate::codebase::dependencies::extract::{ImportedBinding, ImportedBindingKind};
use crate::codebase::dependencies::graph::{DepGraph, ResolvedCallSite, TsFactLookup};
use crate::codebase::ts_resolver::VisiblePathLookup;
use crate::codebase::ts_source::facts::TsFileFacts;
use std::path::{Path, PathBuf};

const HOP_CAP: usize = 8;

pub(super) enum Home {
    Object(PathBuf, String),
    Namespace(PathBuf),
}

pub(super) struct Locate<'a> {
    pub(super) graph: &'a DepGraph,
    pub(super) facts: &'a dyn TsFactLookup,
    pub(super) resolution: &'a super::super::Resolution<'a>,
    pub(super) visible: &'a dyn VisiblePathLookup,
    pub(super) resolver: super::resolve::RegistryResolver<'a>,
}

pub(super) fn homes(locate: &Locate<'_>, site: &ResolvedCallSite, object: &str) -> Vec<Home> {
    let Some(caller) = locate.facts.get_ts_facts(&site.file) else {
        return Vec::new();
    };
    let mut found = Vec::new();
    if local_registry(caller, object) {
        found.push(Home::Object(site.file.clone(), object.to_string()));
    }
    if let Some(binding) = imported_binding(caller, object) {
        if let Some(home) = imported_home(locate, &site.file, binding) {
            found.push(home);
        }
    }
    found
}

pub(super) fn binding_local(
    locate: &Locate<'_>,
    file: &Path,
    name: &str,
    seen: &mut Vec<(PathBuf, String)>,
) -> Option<(PathBuf, String)> {
    if seen.len() >= HOP_CAP
        || seen
            .iter()
            .any(|(seen_file, seen_name)| seen_file == file && seen_name == name)
    {
        return None;
    }
    seen.push((file.to_path_buf(), name.to_string()));
    let ts = locate.facts.get_ts_facts(file)?;
    if let Some(export) = ts
        .exported_bindings
        .iter()
        .find(|export| export.exported == name)
    {
        return named_export(locate, file, export, seen);
    }
    one_star(locate, file, name, &ts.star_reexport_specifiers, seen)
}

pub(super) fn function_exports(locate: &Locate<'_>, file: &Path) -> Vec<(PathBuf, String)> {
    let mut out = Vec::new();
    let mut seen = Vec::new();
    collect_functions(locate, file, &mut seen, &mut out);
    out
}

fn local_registry(caller: &TsFileFacts, object: &str) -> bool {
    let prefix = format!("{object}.");
    caller.callable_aliases.iter().any(|alias| {
        alias.scope_id.is_none()
            && alias.invalidated_at.is_none()
            && alias.local.starts_with(&prefix)
    })
}

fn imported_binding<'a>(caller: &'a TsFileFacts, object: &str) -> Option<&'a ImportedBinding> {
    caller
        .imported_bindings
        .iter()
        .find(|binding| binding.local == object && !binding.is_type_only)
}

fn imported_home(locate: &Locate<'_>, from: &Path, binding: &ImportedBinding) -> Option<Home> {
    let file = super::resolve::resolve(locate, from, &binding.specifier)?;
    match binding.kind {
        ImportedBindingKind::Namespace => Some(Home::Namespace(file)),
        ImportedBindingKind::Named | ImportedBindingKind::Default => {
            let (file, local) = binding_local(locate, &file, &binding.imported, &mut Vec::new())?;
            Some(Home::Object(file, local))
        }
    }
}

fn named_export(
    locate: &Locate<'_>,
    file: &Path,
    export: &crate::codebase::dependencies::extract::ExportedBinding,
    seen: &mut Vec<(PathBuf, String)>,
) -> Option<(PathBuf, String)> {
    if export.local == "*" {
        return None;
    }
    match export.specifier.as_deref() {
        Some(specifier) => {
            let target = super::resolve::resolve(locate, file, specifier)?;
            binding_local(locate, &target, &export.local, seen)
        }
        None => Some((file.to_path_buf(), export.local.clone())),
    }
}

fn one_star(
    locate: &Locate<'_>,
    file: &Path,
    name: &str,
    specifiers: &[String],
    seen: &[(PathBuf, String)],
) -> Option<(PathBuf, String)> {
    let mut found = None;
    for specifier in specifiers {
        let Some(target) = super::resolve::resolve(locate, file, specifier) else {
            continue;
        };
        let Some(hit) = binding_local(locate, &target, name, &mut seen.to_vec()) else {
            continue;
        };
        if found.is_some() {
            return None;
        }
        found = Some(hit);
    }
    found
}

fn collect_functions(
    locate: &Locate<'_>,
    file: &Path,
    seen: &mut Vec<PathBuf>,
    out: &mut Vec<(PathBuf, String)>,
) {
    if seen.len() >= HOP_CAP || seen.iter().any(|item| item == file) {
        return;
    }
    seen.push(file.to_path_buf());
    let Some(ts) = locate.facts.get_ts_facts(file) else {
        return;
    };
    for name in &ts.exported_functions {
        if ts.callable_scope_ids.iter().any(|(_, scope)| scope == name) {
            out.push((file.to_path_buf(), name.clone()));
        }
    }
    for export in &ts.exported_bindings {
        follow_sourced_function(locate, file, export, out);
    }
    for specifier in &ts.star_reexport_specifiers {
        if let Some(target) = super::resolve::resolve(locate, file, specifier) {
            collect_functions(locate, &target, seen, out);
        }
    }
}

fn follow_sourced_function(
    locate: &Locate<'_>,
    file: &Path,
    export: &crate::codebase::dependencies::extract::ExportedBinding,
    out: &mut Vec<(PathBuf, String)>,
) {
    if export.local == "*" {
        return;
    }
    let Some(specifier) = export.specifier.as_deref() else {
        return;
    };
    let Some(target) = super::resolve::resolve(locate, file, specifier) else {
        return;
    };
    let Some((found, local)) = binding_local(locate, &target, &export.local, &mut Vec::new())
    else {
        return;
    };
    let callable = locate.facts.get_ts_facts(&found).is_some_and(|ts| {
        ts.callable_scope_ids
            .iter()
            .any(|(_, scope)| scope == &local)
    });
    if callable {
        out.push((found, local));
    }
}
