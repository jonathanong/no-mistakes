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

enum LookupState {
    External,
    Ambiguous,
    Unresolved,
    Unrecorded,
}

struct LookupQuery<'a> {
    catalog: &'a Catalog<'a>,
    importer: &'a Path,
    specifier: &'a str,
    relative: bool,
    non_source: bool,
    record_gap: bool,
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
    let state = match graph.and_then(|graph| graph.import_target(importer, specifier)) {
        Some(ImportTarget::File(path)) => return is_indexable(path).then(|| path.clone()),
        Some(ImportTarget::External) => LookupState::External,
        Some(ImportTarget::Ambiguous) => LookupState::Ambiguous,
        Some(ImportTarget::Unresolved) => LookupState::Unresolved,
        None => LookupState::Unrecorded,
    };
    let specifier_path = Path::new(specifier);
    let query = LookupQuery {
        catalog,
        importer,
        specifier,
        relative: specifier.starts_with('.') || specifier_path.is_absolute(),
        non_source: has_explicit_non_source_extension(specifier_path),
        record_gap,
    };
    if query.relative && query.non_source {
        return None;
    }
    match state {
        LookupState::External => query.resolve_ambient(),
        LookupState::Ambiguous => {
            if matches!(query.resolve_via_catalog(), Some(Some(path)) if !is_indexable(&path)) {
                return None;
            }
            if query.record_gap {
                query
                    .catalog
                    .record_unresolved(query.importer, query.specifier);
            }
            None
        }
        LookupState::Unresolved => {
            // The graph marks every `#` import as local, even when no mapping exists.
            query.lookup_fallback(!specifier.starts_with('#'))
        }
        LookupState::Unrecorded => query.lookup_fallback(false),
    }
}

impl LookupQuery<'_> {
    fn lookup_fallback(&self, unresolved_target: bool) -> Option<PathBuf> {
        let resolution = self.resolve_via_catalog();
        if let Some(Some(path)) = resolution {
            return is_indexable(&path).then_some(path);
        }
        if self.relative && self.catalog.resolution.is_none() {
            let hits = relative_hits(self.catalog, self.importer, self.specifier);
            if hits.len() == 1 {
                return hits.into_iter().next();
            }
        }
        let ambient = self.catalog.ambient_files(self.specifier);
        if ambient.len() == 1 {
            return ambient.into_iter().next();
        }
        if self.record_gap
            && (!ambient.is_empty()
                || ((unresolved_target || self.relative || resolution.is_some())
                    && !self.non_source))
        {
            self.catalog
                .record_unresolved(self.importer, self.specifier);
        }
        None
    }

    fn resolve_ambient(&self) -> Option<PathBuf> {
        let ambient = self.catalog.ambient_files(self.specifier);
        if ambient.len() == 1 {
            return ambient.into_iter().next();
        }
        if self.record_gap && !ambient.is_empty() {
            self.catalog
                .record_unresolved(self.importer, self.specifier);
        }
        None
    }

    fn resolve_via_catalog(&self) -> Option<Option<PathBuf>> {
        let resolution = self.catalog.resolution?;
        let trusted = self.record_gap
            && self.catalog.tags.iter().any(|tag| {
                !tag.module.is_empty()
                    && !tag.name.is_empty()
                    && super::tags::module_matches(self.specifier, &tag.module)
            });
        resolution.resolve(self.specifier, self.importer, self.relative || trusted)
    }
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
