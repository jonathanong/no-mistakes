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
    use crate::codebase::ts_resolver::{has_explicit_non_source_extension, is_bare_node_builtin};
    if specifier.starts_with("node:") || is_bare_node_builtin(specifier) {
        return None;
    }
    let specifier_path = Path::new(specifier);
    let relative = specifier.starts_with('.') || specifier_path.is_absolute();
    let non_source = has_explicit_non_source_extension(specifier_path);
    match graph.and_then(|graph| graph.import_target(importer, specifier)) {
        Some(ImportTarget::File(path)) => is_indexable(path).then(|| path.clone()),
        Some(ImportTarget::External) => {
            if relative && non_source {
                return None;
            }
            resolve_ambient(catalog, importer, specifier, record_gap)
        }
        Some(ImportTarget::Ambiguous) => {
            if relative && non_source {
                return None;
            }
            if matches!(
                resolve_with_catalog(catalog, importer, specifier, relative, record_gap),
                Some(Some(path)) if !is_indexable(&path)
            ) {
                return None;
            }
            if record_gap {
                catalog.record_unresolved(importer, specifier);
            }
            None
        }
        Some(ImportTarget::Unresolved) => resolve_local_or_unrecorded(
            catalog,
            importer,
            specifier,
            relative,
            non_source,
            record_gap,
            !specifier.starts_with('#'),
        ),
        None => resolve_local_or_unrecorded(
            catalog, importer, specifier, relative, non_source, record_gap, false,
        ),
    }
}

fn resolve_with_catalog(
    catalog: &Catalog<'_>,
    importer: &Path,
    specifier: &str,
    relative: bool,
    record_gap: bool,
) -> Option<Option<PathBuf>> {
    let trusted = catalog.tags.iter().any(|tag| {
        !tag.module.is_empty()
            && !tag.name.is_empty()
            && super::tags::module_matches(specifier, &tag.module)
    });
    catalog
        .resolution?
        .resolve(specifier, importer, relative || (trusted && record_gap))
}

fn resolve_ambient(
    catalog: &Catalog<'_>,
    importer: &Path,
    specifier: &str,
    record_gap: bool,
) -> Option<PathBuf> {
    let ambient = catalog.ambient_files(specifier);
    if ambient.len() == 1 {
        return ambient.into_iter().next();
    }
    if record_gap && !ambient.is_empty() {
        catalog.record_unresolved(importer, specifier);
    }
    None
}

fn resolve_local_or_unrecorded(
    catalog: &Catalog<'_>,
    importer: &Path,
    specifier: &str,
    relative: bool,
    non_source: bool,
    record_gap: bool,
    graph_unresolved: bool,
) -> Option<PathBuf> {
    if relative && non_source {
        return None;
    }
    let resolution = resolve_with_catalog(catalog, importer, specifier, relative, record_gap);
    if let Some(Some(path)) = &resolution {
        return is_indexable(path).then(|| path.clone());
    }
    if relative && catalog.resolution.is_none() {
        let hits = relative_hits(catalog, importer, specifier);
        if hits.len() == 1 {
            return hits.into_iter().next();
        }
    }
    let ambient = catalog.ambient_files(specifier);
    if ambient.len() == 1 {
        return ambient.into_iter().next();
    }
    if record_gap
        && (!ambient.is_empty()
            || ((graph_unresolved || relative || resolution.is_some()) && !non_source))
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

pub(super) use super::ambient::declare_module_specifiers;
