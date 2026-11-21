use super::fragments::{Catalog, Fragment};
use super::source_text::file_text;
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::dependencies::extract::{ImportedBinding, ImportedBindingKind};
use crate::codebase::dependencies::graph::{DepGraph, ImportTarget, TsFactLookup};
use std::path::{Path, PathBuf};

pub(super) enum ModuleHit {
    File(PathBuf),
    /// A builtin or package that cannot import the trusted SQL tag.
    External,
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
            ImportTarget::External => return ambient_or_external(facts, catalog, specifier),
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
        ModuleHit::External | ModuleHit::Open => None,
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
        ModuleHit::External => Vec::new(),
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

fn ambient_or_external(facts: &CheckFactMap, catalog: &Catalog<'_>, specifier: &str) -> ModuleHit {
    let mut hits = ambient_hits(facts, catalog, specifier);
    hits.sort();
    hits.dedup();
    match hits.len() {
        1 => ModuleHit::File(hits.remove(0)),
        0 => ModuleHit::External,
        _ => ModuleHit::Open,
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
    facts
        .graph_file_universe()
        .iter()
        .filter(|file| {
            facts
                .get_ts_facts(file)
                .and_then(|ts| file_text(ts, catalog.sources, file))
                .is_some_and(|source| declares_ambient_module(&source, specifier))
        })
        .cloned()
        .collect()
}

/// `declare module "specifier"` with any whitespace between the tokens.
fn declares_ambient_module(source: &str, specifier: &str) -> bool {
    let double = format!("\"{specifier}\"");
    let single = format!("'{specifier}'");
    let mut index = 0;
    while let Some(found) = source[index..].find("declare") {
        let at = index + found;
        let boundary = at == 0 || !is_ident_byte(source.as_bytes()[at - 1]);
        let after = at + "declare".len();
        if boundary && module_name_follows(&source[after..], &double, &single) {
            return true;
        }
        index = after;
    }
    false
}

fn module_name_follows(after_declare: &str, double: &str, single: &str) -> bool {
    let trimmed = after_declare.trim_start();
    if trimmed.len() == after_declare.len() {
        return false;
    }
    let Some(after_keyword) = trimmed.strip_prefix("module") else {
        return false;
    };
    if after_keyword.starts_with(|ch: char| is_ident_char(ch)) {
        return false;
    }
    let quoted = after_keyword.trim_start();
    if quoted.len() == after_keyword.len() {
        return false;
    }
    quoted.starts_with(double) || quoted.starts_with(single)
}

fn is_ident_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'$'
}

fn is_ident_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_' || ch == '$'
}
