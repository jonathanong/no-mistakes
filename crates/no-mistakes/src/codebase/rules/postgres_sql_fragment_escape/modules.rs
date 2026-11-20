use super::fragments::{Catalog, Fragment};
use super::source_text::file_text;
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::dependencies::extract::{ImportedBinding, ImportedBindingKind};
use crate::codebase::dependencies::graph::{DepGraph, ImportTarget, TsFactLookup};
use std::path::{Path, PathBuf};

pub(super) enum ModuleHit {
    File(PathBuf),
    Open,
}

pub(super) fn module_hit(
    graph: &DepGraph,
    facts: &CheckFactMap,
    catalog: &Catalog<'_>,
    importer: &Path,
    specifier: &str,
) -> ModuleHit {
    if let Some(recorded) = graph.import_target(importer, specifier) {
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
        hits.extend(ambient_hits(facts, catalog, specifier));
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
    graph: &DepGraph,
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
    graph: &DepGraph,
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

fn ambient_hits(facts: &CheckFactMap, catalog: &Catalog<'_>, specifier: &str) -> Vec<PathBuf> {
    let double = format!("declare module \"{specifier}\"");
    let single = format!("declare module '{specifier}'");
    facts
        .graph_file_universe()
        .iter()
        .filter(|file| {
            facts
                .get_ts_facts(file)
                .and_then(|ts| file_text(ts, catalog.sources, file))
                .is_some_and(|source| source.contains(&double) || source.contains(&single))
        })
        .cloned()
        .collect()
}
