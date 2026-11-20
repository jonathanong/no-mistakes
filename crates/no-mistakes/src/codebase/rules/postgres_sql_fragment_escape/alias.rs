use super::fragments::{Catalog, Fragment};
use super::modules::module_file;
use super::source_text::file_text;
use super::text::{export_alias_local, import_specifier};
use super::visible::visible;
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::dependencies::graph::DepGraph;
use crate::codebase::ts_source::facts::TsFileFacts;
use std::path::{Path, PathBuf};

pub(super) fn hop(
    catalog: &Catalog,
    graph: &DepGraph,
    facts: &CheckFactMap,
    module: &Path,
    spec: &str,
    name: &str,
    stack: &mut Vec<(PathBuf, String)>,
) -> Vec<Fragment> {
    if let Some(next) = module_file(graph, facts, catalog, module, spec) {
        visible(catalog, graph, facts, &next, name, stack)
    } else {
        unique_export(catalog, name)
    }
}

pub(super) fn source_alias(
    catalog: &Catalog,
    graph: &DepGraph,
    facts: &CheckFactMap,
    module: &Path,
    ts: &TsFileFacts,
    name: &str,
    stack: &mut Vec<(PathBuf, String)>,
) -> Vec<Fragment> {
    let owned = file_text(ts, catalog.sources, module);
    let Some(source) = owned.as_deref() else {
        return Vec::new();
    };
    let Some(local) = export_alias_local(source, name) else {
        return Vec::new();
    };
    let Some(spec) = import_specifier(source, local) else {
        return Vec::new();
    };
    hop(catalog, graph, facts, module, spec, local, stack)
}

fn unique_export(catalog: &Catalog, name: &str) -> Vec<Fragment> {
    let found = catalog.by_export(name);
    if found.len() == 1 {
        found
    } else {
        Vec::new()
    }
}
