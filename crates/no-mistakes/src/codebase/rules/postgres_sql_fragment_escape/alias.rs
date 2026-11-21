use super::fragments::{Catalog, Fragment};
use super::modules::{module_hit, ModuleHit};
use super::source_text::file_text;
use super::text::{export_alias_local, import_specifier};
use super::visible::visible;
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::dependencies::graph::{DepGraph, TsFactLookup};
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
    match module_hit(graph, facts, catalog, module, spec) {
        ModuleHit::File(next) => visible(catalog, graph, facts, &next, name, stack),
        ModuleHit::External => Vec::new(),
        ModuleHit::Open => vec![Fragment::open(spec)],
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

pub(super) fn star_fragments(
    catalog: &Catalog,
    graph: &DepGraph,
    facts: &CheckFactMap,
    module: &Path,
    blocked: &[String],
    stack: &mut Vec<PathBuf>,
) -> Vec<Fragment> {
    if stack.iter().any(|seen| seen == module) {
        return Vec::new();
    }
    let Some(ts) = facts.get_ts_facts(module) else {
        return Vec::new();
    };
    stack.push(module.to_path_buf());
    let mut found = Vec::new();
    for binding in &ts.exported_bindings {
        if binding.exported == "default" || blocked.iter().any(|name| name == &binding.exported) {
            continue;
        }
        found.extend(visible(
            catalog,
            graph,
            facts,
            module,
            &binding.exported,
            &mut Vec::new(),
        ));
    }
    let mut next_blocked = blocked.to_vec();
    next_blocked.extend(
        ts.exported_bindings
            .iter()
            .map(|binding| binding.exported.clone()),
    );
    for spec in &ts.star_reexport_specifiers {
        match module_hit(graph, facts, catalog, module, spec) {
            ModuleHit::File(next) => found.extend(star_fragments(
                catalog,
                graph,
                facts,
                &next,
                &next_blocked,
                stack,
            )),
            ModuleHit::External => {}
            ModuleHit::Open => found.push(Fragment::open(spec)),
        }
    }
    stack.pop();
    found
}
