use super::fragments::{Catalog, Fragment};
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::dependencies::extract::{ImportedBinding, ImportedBindingKind};
use crate::codebase::dependencies::graph::{DepGraph, ImportTarget};
use std::path::{Path, PathBuf};

pub(super) enum ModuleHit {
    File(PathBuf),
    Open,
}

pub(super) fn module_hit(
    graph: Option<&DepGraph>,
    _facts: &CheckFactMap,
    catalog: &Catalog<'_>,
    importer: &Path,
    specifier: &str,
) -> ModuleHit {
    if let Some(recorded) = graph.and_then(|graph| graph.import_target(importer, specifier)) {
        match recorded {
            ImportTarget::File(path) => return ModuleHit::File(path.clone()),
            ImportTarget::Ambiguous => return ModuleHit::Open,
            ImportTarget::Unresolved => {}
        }
    }
    if let Some(path) = relative_module(catalog, importer, specifier) {
        return ModuleHit::File(path);
    }
    let mut hits = Vec::new();
    if specifier.starts_with('.') {
        hits.extend(relative_hits(catalog, importer, specifier));
    }
    if hits.is_empty() {
        hits.extend(catalog.ambient_files(specifier));
    }
    hits.sort();
    hits.dedup();
    if hits.len() == 1 {
        ModuleHit::File(hits.remove(0))
    } else {
        ModuleHit::Open
    }
}

pub(super) fn module_file(
    graph: Option<&DepGraph>,
    facts: &CheckFactMap,
    catalog: &Catalog<'_>,
    importer: &Path,
    specifier: &str,
) -> Option<PathBuf> {
    match module_hit(graph, facts, catalog, importer, specifier) {
        ModuleHit::File(path) => Some(path),
        ModuleHit::Open => None,
    }
}

pub(super) fn import_hit_fragments(
    catalog: &Catalog<'_>,
    graph: Option<&DepGraph>,
    facts: &CheckFactMap,
    importer: &Path,
    import: &ImportedBinding,
) -> Vec<Fragment> {
    match module_hit(graph, facts, catalog, importer, &import.specifier) {
        ModuleHit::Open => vec![Fragment::open(&import.specifier)],
        ModuleHit::File(module) if import.kind == ImportedBindingKind::Namespace => {
            super::visible::visible_all(catalog, graph, facts, &module, &mut Vec::new())
        }
        ModuleHit::File(module) => {
            let name = super::tags::exported_name(import);
            super::visible::visible(catalog, graph, facts, &module, &name, &mut Vec::new())
        }
    }
}

fn relative_module(catalog: &Catalog<'_>, importer: &Path, specifier: &str) -> Option<PathBuf> {
    if !specifier.starts_with('.') {
        return None;
    }
    let mut hits = relative_hits(catalog, importer, specifier);
    (hits.len() == 1).then(|| hits.pop()).flatten()
}

fn relative_hits(catalog: &Catalog<'_>, importer: &Path, specifier: &str) -> Vec<PathBuf> {
    let Some(dir) = importer.parent() else {
        return Vec::new();
    };
    let logical = crate::codebase::ts_resolver::normalize_path(&dir.join(specifier));
    let Some(stem) = logical.file_stem().and_then(|stem| stem.to_str()) else {
        return Vec::new();
    };
    let Some(parent) = logical.parent() else {
        return Vec::new();
    };
    catalog.relative_candidates(parent, stem)
}

pub(super) fn declare_module_specifiers(source: &str) -> Vec<String> {
    let mut found = Vec::new();
    push_declare_modules(source, "declare module \"", '"', &mut found);
    push_declare_modules(source, "declare module '", '\'', &mut found);
    found.sort();
    found.dedup();
    found
}

fn push_declare_modules(source: &str, marker: &str, quote: char, found: &mut Vec<String>) {
    let mut rest = source;
    while let Some(index) = rest.find(marker) {
        let after = &rest[index + marker.len()..];
        if let Some(end) = after.find(quote) {
            if !after[..end].is_empty() {
                found.push(after[..end].to_string());
            }
        }
        rest = &rest[index + marker.len()..];
    }
}
