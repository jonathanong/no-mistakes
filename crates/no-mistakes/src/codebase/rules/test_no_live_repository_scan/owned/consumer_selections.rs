use super::route_index::RouteHit;
use super::{Consumed, Gate};
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::codebase::ts_source::module_facts::arguments::{PathProvenance, RouteKind};
use crate::fx::FxHashSet;
use std::path::Path;

pub(super) fn collect(
    gate: &Gate<'_>,
    consumed: &mut Consumed,
    file: &Path,
    ts: &TsFileFacts,
    provenance: &PathProvenance,
    specifier: &str,
    target: &Path,
) -> FxHashSet<u32> {
    let mut selected = FxHashSet::default();
    let Some(module) = ts.module_bindings.as_deref() else {
        return selected;
    };
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
        if names.is_empty() {
            selected.insert(site.start);
            continue;
        }
        let Some((graph, facts)) = gate.selection else {
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
            .map(|name| graph.file_export_node(target, name))
            .collect::<Option<Vec<_>>>();
        let Some(targets) = targets else {
            continue;
        };
        for node in targets {
            if let Some((target, symbol)) = node.as_symbol() {
                consumed
                    .hits
                    .push(RouteHit::selected(file, site, target, symbol.to_string()));
            }
        }
        selected.insert(site.start);
    }
    selected
}
