use super::route_index::RouteHit;
use super::Consumed;
use crate::codebase::dependencies::graph::{DepGraph, TsFactLookup};
use crate::codebase::ts_source::module_facts::arguments::{PathProvenance, RouteKind};
use crate::codebase::ts_source::module_facts::TypeScriptModuleFacts;
use crate::fx::FxHashSet;
use std::path::Path;

pub(super) fn collect(
    context: (&DepGraph, &dyn TsFactLookup),
    consumed: &mut Consumed,
    file: &Path,
    module: &TypeScriptModuleFacts,
    provenance: &PathProvenance,
    specifier: &str,
    target: &Path,
) -> FxHashSet<u32> {
    let mut selected = FxHashSet::default();
    let (graph, facts) = context;
    for site in provenance
        .route_sites
        .iter()
        .filter(|site| site.kind == RouteKind::Dynamic && site.detail == specifier)
    {
        if module.discarded_dynamic_imports.contains(&site.start) {
            selected.insert(site.start);
            continue;
        }
        let Some((_, names, calls)) = module
            .dynamic_import_selectors
            .iter()
            .find(|(offset, _, _)| *offset == site.start)
        else {
            continue;
        };
        if !calls
            .iter()
            .all(|name| graph.file_export_receiver_safe(target, name, facts))
        {
            continue;
        }
        let targets = names
            .iter()
            .map(|name| graph.file_export_binding(target, name))
            .collect::<Option<Vec<_>>>();
        let Some(targets) = targets else {
            continue;
        };
        for (target, symbol) in targets {
            consumed
                .hits
                .push(RouteHit::selected(file, site, target, symbol.to_string()));
        }
        selected.insert(site.start);
    }
    selected
}
