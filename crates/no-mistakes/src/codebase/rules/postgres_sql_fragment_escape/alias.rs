use super::fragments::{Catalog, Fragment};
use super::modules::{module_hit, ModuleHit};
use super::source_text::file_text;
use super::tags::{exported_name, imported};
use super::text::{export_alias_local, import_specifier};
use super::visible::{visible, visible_all};
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::dependencies::extract::ImportedBindingKind;
use crate::codebase::dependencies::graph::{DepGraph, TsFactLookup};
use crate::codebase::ts_source::facts::TsFileFacts;
use std::path::{Path, PathBuf};

pub(super) fn hop(
    catalog: &Catalog,
    graph: Option<&DepGraph>,
    facts: &CheckFactMap,
    module: &Path,
    spec: &str,
    name: &str,
    stack: &mut Vec<(PathBuf, String)>,
) -> Vec<Fragment> {
    match module_hit(graph, facts, catalog, module, spec) {
        ModuleHit::File(next) => visible(catalog, graph, facts, &next, name, stack),
        ModuleHit::Open => vec![Fragment::open(spec)],
        ModuleHit::Unproven => Vec::new(),
    }
}

pub(super) fn source_alias(
    catalog: &Catalog,
    graph: Option<&DepGraph>,
    facts: &CheckFactMap,
    module: &Path,
    ts: &TsFileFacts,
    name: &str,
    stack: &mut Vec<(PathBuf, String)>,
) -> Vec<Fragment> {
    if let Some(found) = alias_from_facts(catalog, graph, facts, module, ts, name, stack) {
        return found;
    }
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
    if star_import(source, local) {
        return reexported_namespace(catalog, graph, facts, module, spec);
    }
    hop(catalog, graph, facts, module, spec, local, stack)
}

fn alias_from_facts(
    catalog: &Catalog,
    graph: Option<&DepGraph>,
    facts: &CheckFactMap,
    module: &Path,
    ts: &TsFileFacts,
    name: &str,
    stack: &mut Vec<(PathBuf, String)>,
) -> Option<Vec<Fragment>> {
    let export = ts.exported_bindings.iter().find(|binding| {
        binding.specifier.is_none() && binding.exported == name && binding.local != name
    })?;
    let import = imported(ts, &export.local)?;
    if import.kind == ImportedBindingKind::Namespace {
        return Some(reexported_namespace(
            catalog,
            graph,
            facts,
            module,
            &import.specifier,
        ));
    }
    Some(hop(
        catalog,
        graph,
        facts,
        module,
        &import.specifier,
        &exported_name(import),
        stack,
    ))
}

fn reexported_namespace(
    catalog: &Catalog,
    graph: Option<&DepGraph>,
    facts: &CheckFactMap,
    module: &Path,
    spec: &str,
) -> Vec<Fragment> {
    match module_hit(graph, facts, catalog, module, spec) {
        ModuleHit::File(next) => visible_all(catalog, graph, facts, &next, &mut Vec::new()),
        ModuleHit::Open => vec![Fragment::open(spec)],
        ModuleHit::Unproven => Vec::new(),
    }
}

fn star_import(source: &str, local: &str) -> bool {
    let marker = format!("import * as {local}");
    source.lines().any(|line| {
        let line = line.trim();
        line.starts_with(&marker)
            && line[marker.len()..]
                .chars()
                .next()
                .is_some_and(|ch| ch.is_whitespace())
    })
}

pub(super) fn star_fragments(
    catalog: &Catalog,
    graph: Option<&DepGraph>,
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
            ModuleHit::Open => found.push(Fragment::open(spec)),
            ModuleHit::Unproven => {}
        }
    }
    stack.pop();
    found
}
