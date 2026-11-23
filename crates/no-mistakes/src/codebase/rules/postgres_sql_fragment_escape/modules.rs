use super::fragments::{Catalog, Fragment};
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::dependencies::extract::{is_indexable, ImportedBinding, ImportedBindingKind};
use crate::codebase::dependencies::graph::{DepGraph, ImportTarget};
use std::path::{Path, PathBuf};

pub(super) fn module_file(
    graph: Option<&DepGraph>,
    _facts: &CheckFactMap,
    catalog: &Catalog<'_>,
    importer: &Path,
    specifier: &str,
) -> Option<PathBuf> {
    module_lookup(graph, catalog, importer, specifier, true)
}

pub(super) fn module_lookup(
    graph: Option<&DepGraph>,
    catalog: &Catalog<'_>,
    importer: &Path,
    specifier: &str,
    record_gap: bool,
) -> Option<PathBuf> {
    use crate::codebase::ts_resolver::is_bare_node_builtin;
    if specifier.starts_with("node:") || is_bare_node_builtin(specifier) {
        return None;
    }
    let recorded = graph.and_then(|graph| graph.import_target(importer, specifier));
    if let Some(ImportTarget::File(path)) = recorded {
        return is_indexable(path).then(|| path.clone());
    }
    let relative = specifier.starts_with('.') || Path::new(specifier).is_absolute();
    if relative
        && crate::codebase::ts_resolver::has_explicit_non_source_extension(Path::new(specifier))
    {
        return None;
    }
    let trusted = catalog.tags.iter().any(|tag| {
        !tag.module.is_empty()
            && !tag.name.is_empty()
            && super::tags::module_matches(specifier, &tag.module)
    });
    let ambient = catalog.ambient_files(specifier);
    let external = recorded == Some(&ImportTarget::External);
    let resolution = catalog
        .resolution
        .filter(|_| !external)
        .and_then(|resolution| {
            resolution.resolve(specifier, importer, relative || (trusted && record_gap))
        });
    let ambiguous = recorded == Some(&ImportTarget::Ambiguous);
    if let Some(Some(path)) = &resolution {
        if !is_indexable(path) {
            return None;
        }
        if !ambiguous {
            return Some(path.clone());
        }
    }
    if !ambiguous {
        if !external && relative && catalog.resolution.is_none() {
            let hits = relative_hits(catalog, importer, specifier);
            if hits.len() == 1 {
                return hits.into_iter().next();
            }
        }
        if ambient.len() == 1 {
            return ambient.into_iter().next();
        }
    }
    let unresolved = recorded == Some(&ImportTarget::Unresolved);
    if record_gap
        && (ambiguous
            || unresolved
            || !ambient.is_empty()
            || (!external && relative)
            || resolution.is_some())
    {
        catalog.record_unresolved(importer, specifier);
    }
    None
}

pub(super) fn import_hit_fragments(
    catalog: &Catalog<'_>,
    graph: Option<&DepGraph>,
    facts: &CheckFactMap,
    importer: &Path,
    import: &ImportedBinding,
    is_tag: bool,
) -> Vec<Fragment> {
    let Some(module) = module_lookup(graph, catalog, importer, &import.specifier, !is_tag) else {
        return Vec::new();
    };
    if import.kind == ImportedBindingKind::Namespace {
        super::visible::visible_all(catalog, graph, facts, &module, &mut Vec::new())
    } else {
        let name = super::tags::exported_name(import);
        super::visible::visible(catalog, graph, facts, &module, &name, &mut Vec::new())
    }
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
    let mut index = 0;
    while let Some(offset) = source[index..].find("declare") {
        let at = index + offset;
        let boundary = at == 0 || !is_ident_byte(source.as_bytes()[at - 1]);
        let after = at + "declare".len();
        if boundary {
            if let Some(name) = module_name_follows(&source[after..]) {
                found.push(name.to_owned());
            }
        }
        index = after;
    }
    found.sort();
    found.dedup();
    found
}

fn module_name_follows(after_declare: &str) -> Option<&str> {
    let trimmed = after_declare.trim_start();
    if trimmed.len() == after_declare.len() {
        return None;
    }
    let after_keyword = trimmed.strip_prefix("module")?;
    if after_keyword.starts_with(|ch: char| is_ident_char(ch)) {
        return None;
    }
    let quoted = after_keyword.trim_start();
    if quoted.len() == after_keyword.len() {
        return None;
    }
    let quote = quoted.chars().next()?;
    if !matches!(quote, '\'' | '"') {
        return None;
    }
    let rest = &quoted[1..];
    let name = &rest[..rest.find(quote)?];
    (!name.is_empty()).then_some(name)
}

fn is_ident_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'$'
}

fn is_ident_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_' || ch == '$'
}
