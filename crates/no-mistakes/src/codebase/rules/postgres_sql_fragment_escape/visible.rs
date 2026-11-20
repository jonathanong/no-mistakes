use super::alias::{hop, source_alias};
use super::fragments::{dedup, Catalog, Fragment};
use super::modules::module_file;
use super::tags::{exported_name, imported};
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::dependencies::extract::ExportedBinding;
use crate::codebase::dependencies::graph::{DepGraph, TsFactLookup};
use crate::codebase::ts_source::facts::TsFileFacts;
use std::path::{Path, PathBuf};

pub(super) fn visible(
    catalog: &Catalog,
    graph: &DepGraph,
    facts: &CheckFactMap,
    module: &Path,
    name: &str,
    stack: &mut Vec<(PathBuf, String)>,
) -> Vec<Fragment> {
    let key = (module.to_path_buf(), name.to_string());
    if stack.contains(&key) {
        return Vec::new();
    }
    stack.push(key);
    let mut found = catalog.local(module, name);
    if found.is_empty() {
        found.extend(follow(catalog, graph, facts, module, name, stack));
    }
    stack.pop();
    dedup(&mut found);
    found
}

pub(super) fn visible_all(
    catalog: &Catalog,
    graph: &DepGraph,
    facts: &CheckFactMap,
    module: &Path,
    stack: &mut Vec<PathBuf>,
) -> Vec<Fragment> {
    if stack.iter().any(|seen| seen == module) {
        return Vec::new();
    }
    stack.push(module.to_path_buf());
    let mut found = catalog.local_all(module);
    if let Some(ts) = facts.get_ts_facts(module) {
        let explicit: Vec<&str> = ts
            .exported_bindings
            .iter()
            .map(|binding| binding.exported.as_str())
            .collect();
        for spec in &ts.star_reexport_specifiers {
            if let Some(next) = module_file(graph, facts, catalog, module, spec) {
                found.extend(
                    visible_all(catalog, graph, facts, &next, stack)
                        .into_iter()
                        .filter(|fragment| {
                            fragment.exported != "default"
                                && !explicit.contains(&fragment.exported.as_str())
                        }),
                );
            }
        }
        for binding in &ts.exported_bindings {
            found.extend(relay_all(catalog, graph, facts, module, ts, binding));
        }
    }
    stack.pop();
    dedup(&mut found);
    found
}

fn follow(
    catalog: &Catalog,
    graph: &DepGraph,
    facts: &CheckFactMap,
    module: &Path,
    name: &str,
    stack: &mut Vec<(PathBuf, String)>,
) -> Vec<Fragment> {
    let Some(ts) = facts.get_ts_facts(module) else {
        return Vec::new();
    };
    let mut found = Vec::new();
    for binding in &ts.exported_bindings {
        if binding.exported == name {
            found.extend(relay_one(catalog, graph, facts, module, ts, binding, stack));
        }
    }
    let shadowed = name == "default"
        || ts
            .exported_bindings
            .iter()
            .any(|binding| binding.exported == name);
    if !shadowed {
        for spec in &ts.star_reexport_specifiers {
            if let Some(next) = module_file(graph, facts, catalog, module, spec) {
                found.extend(visible(catalog, graph, facts, &next, name, stack));
            }
        }
    }
    if found.is_empty() {
        found.extend(source_alias(catalog, graph, facts, module, ts, name, stack));
    }
    found
}

fn relay_all(
    catalog: &Catalog,
    graph: &DepGraph,
    facts: &CheckFactMap,
    module: &Path,
    ts: &TsFileFacts,
    binding: &ExportedBinding,
) -> Vec<Fragment> {
    if let Some(spec) = &binding.specifier {
        return hop(
            catalog,
            graph,
            facts,
            module,
            spec,
            &binding.local,
            &mut Vec::new(),
        );
    }
    imported(ts, &binding.local)
        .map(|import| {
            hop(
                catalog,
                graph,
                facts,
                module,
                &import.specifier,
                &exported_name(import),
                &mut Vec::new(),
            )
        })
        .unwrap_or_default()
}

fn relay_one(
    catalog: &Catalog,
    graph: &DepGraph,
    facts: &CheckFactMap,
    module: &Path,
    ts: &TsFileFacts,
    binding: &ExportedBinding,
    stack: &mut Vec<(PathBuf, String)>,
) -> Vec<Fragment> {
    if let Some(spec) = &binding.specifier {
        return hop(catalog, graph, facts, module, spec, &binding.local, stack);
    }
    imported(ts, &binding.local)
        .map(|import| {
            hop(
                catalog,
                graph,
                facts,
                module,
                &import.specifier,
                &exported_name(import),
                stack,
            )
        })
        .unwrap_or_default()
}
