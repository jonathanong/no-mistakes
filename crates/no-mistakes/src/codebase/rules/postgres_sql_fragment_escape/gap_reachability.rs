use super::fragments::Catalog;
use super::modules::module_lookup;
use super::relevance::reaches_fragment;
use super::tags::has_tagged_query;
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::dependencies::extract::ImportKind;
use crate::codebase::dependencies::graph::{DepGraph, TsFactLookup};
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::codebase::ts_source::relative_slash_path;
use crate::fx::{fx_set, FxHashSet};
use rayon::prelude::*;
use std::path::{Path, PathBuf};

pub(super) fn relevant_files(
    root: &Path,
    catalog: &Catalog<'_>,
    graph: Option<&DepGraph>,
    facts: &CheckFactMap,
    proved: &FxHashSet<&str>,
) -> FxHashSet<PathBuf> {
    let mut pending: Vec<_> = facts
        .graph_file_universe()
        .par_iter()
        .filter(|file| {
            facts.get_ts_facts(file).is_some_and(|ts| {
                (!proved.is_empty() && proved.contains(relative_slash_path(root, file).as_str()))
                    || (!ts.fatal_parse_error
                        && (has_tagged_query(ts, catalog.tags)
                            || reaches_fragment(catalog, graph, facts, file)))
            })
        })
        .cloned()
        .collect();
    let mut visited = fx_set();
    while let Some(file) = pending.pop() {
        if visited.contains(&file) {
            continue;
        }
        if let Some(ts) = facts.get_ts_facts(&file) {
            let mut specifiers = fx_set();
            for specifier in
                dependency_specifiers(ts).filter(|specifier| specifiers.insert(*specifier))
            {
                if let Some(target) = module_lookup(graph, catalog, &file, specifier, false) {
                    pending.push(target);
                }
            }
        }
        visited.insert(file);
    }
    visited
}

fn dependency_specifiers(ts: &TsFileFacts) -> impl Iterator<Item = &str> {
    ts.imports
        .iter()
        .filter(|item| {
            !item.computed && !matches!(item.kind, ImportKind::Type | ImportKind::RequireResolve)
        })
        .map(|item| item.specifier.as_str())
        .chain(
            ts.namespaces
                .created_require_specifiers
                .iter()
                .map(String::as_str),
        )
        .chain(
            ts.imported_bindings
                .iter()
                .chain(&ts.value_flow.commonjs_imported_bindings)
                .filter(|binding| !binding.is_type_only)
                .map(|binding| binding.specifier.as_str()),
        )
        .chain(
            ts.exported_bindings
                .iter()
                .filter_map(|item| item.specifier.as_deref()),
        )
        .chain(ts.star_reexport_specifiers.iter().map(String::as_str))
}
