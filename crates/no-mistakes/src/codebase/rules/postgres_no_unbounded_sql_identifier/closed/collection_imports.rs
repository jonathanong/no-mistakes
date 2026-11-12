use super::*;
use crate::codebase::dependencies::extract::{
    CatalogRead, CatalogReadKind, ImportedBindingKind, ValueFlowFacts,
};
use crate::codebase::dependencies::graph::{EdgeKind, NodeId};
use crate::codebase::rules::postgres_sql_cross_file::{possible_exports, UniqueExport};
use crate::fx::{FxHashMap, FxHashSet};
use std::path::PathBuf;

pub(super) fn safe(
    graph: &DepGraph,
    facts: &CheckFactMap,
    _bindings: &Bindings<'_>,
    producer: &UniqueExport,
) -> bool {
    let Some(ts) = facts.get_ts_facts(&producer.producer) else {
        return false;
    };
    if !complete(ts) || !name_safe(&ts.value_flow, &producer.exported) {
        return false;
    }
    let mut candidates = FxHashMap::default();
    let mut reachability = FxHashMap::default();
    for file in facts.graph_file_universe() {
        let Some(ts) = facts.get_ts_facts(file) else {
            continue;
        };
        if !complete(ts) {
            if reaches(graph, file, &producer.producer) {
                return false;
            }
            continue;
        }
        let mut reads_by_name: FxHashMap<&str, Vec<&CatalogRead>> = FxHashMap::default();
        for read in ts
            .value_flow
            .catalogs
            .reads
            .values()
            .filter(|read| read.scope == 0)
        {
            reads_by_name.entry(&read.name).or_default().push(read);
        }
        let writes: FxHashSet<_> = ts
            .value_flow
            .identifier_writes
            .iter()
            .filter(|write| write.scope == 0)
            .map(|write| write.name.as_str())
            .collect();
        for import in ts
            .imported_bindings
            .iter()
            .filter(|import| !import.is_type_only)
        {
            let reads = reads_by_name
                .get(import.local.as_str())
                .map_or(&[][..], Vec::as_slice);
            if import.kind == ImportedBindingKind::Namespace {
                if ts
                    .exported_bindings
                    .iter()
                    .any(|export| export.specifier.is_none() && export.local == import.local)
                    && *reachability
                        .entry(file.clone())
                        .or_insert_with(|| reaches(graph, file, &producer.producer))
                {
                    return false;
                }
                for read in reads {
                    let Some((member, path)) = read.path.split_first() else {
                        if *reachability
                            .entry(file.clone())
                            .or_insert_with(|| reaches(graph, file, &producer.producer))
                        {
                            return false;
                        }
                        continue;
                    };
                    if matches_producer(graph, facts, file, member, producer, &mut candidates)
                        && (!read_safe(read, path) || writes.contains(import.local.as_str()))
                    {
                        return false;
                    }
                }
            } else if matches_producer(
                graph,
                facts,
                file,
                &import.imported,
                producer,
                &mut candidates,
            ) && (writes.contains(import.local.as_str())
                || !reads.iter().all(|read| read_safe(read, &read.path))
                || ts
                    .exported_bindings
                    .iter()
                    .any(|export| export.specifier.is_none() && export.local == import.local))
            {
                return false;
            }
        }
    }
    true
}

fn complete(ts: &crate::codebase::ts_source::facts::TsFileFacts) -> bool {
    ts.parse_error.is_none()
        && ts.operational_error.is_none()
        && ts.value_flow.incomplete_reason.is_none()
}

fn name_safe(flow: &ValueFlowFacts, name: &str) -> bool {
    !written(flow, name)
        && flow
            .catalogs
            .reads
            .values()
            .filter(|read| read.scope == 0 && read.name == name)
            .all(|read| read_safe(read, &read.path))
}

fn written(flow: &ValueFlowFacts, name: &str) -> bool {
    flow.identifier_writes
        .iter()
        .any(|write| write.scope == 0 && write.name == name)
}

fn read_safe(read: &CatalogRead, path: &[String]) -> bool {
    match path {
        [member] if member == "length" => read.kind == CatalogReadKind::Value,
        [member] if matches!(member.as_str(), "includes" | "has") => {
            read.kind == CatalogReadKind::Invalid
        }
        _ => false,
    }
}

fn matches_producer(
    graph: &DepGraph,
    facts: &CheckFactMap,
    file: &Path,
    name: &str,
    producer: &UniqueExport,
    cache: &mut FxHashMap<(PathBuf, String), bool>,
) -> bool {
    *cache
        .entry((file.to_path_buf(), name.into()))
        .or_insert_with(|| {
            possible_exports(graph, facts, file, name)
                .iter()
                .any(|candidate| candidate == producer)
        })
}

fn reaches(graph: &DepGraph, file: &Path, producer: &Path) -> bool {
    let mut pending = vec![file.to_path_buf()];
    let mut visited = FxHashSet::default();
    while let Some(file) = pending.pop() {
        if file == producer {
            return true;
        }
        if !visited.insert(file.clone()) {
            continue;
        }
        if let Some(edges) = graph.dependencies_of_node(&NodeId::file(&file)) {
            for (target, kind) in edges {
                if matches!(kind, EdgeKind::Import | EdgeKind::Require) {
                    if let Some(file) = target.as_file() {
                        pending.push(file.to_path_buf());
                    }
                }
            }
        }
    }
    false
}

#[cfg(test)]
#[path = "collection_imports_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "collection_imports/real_source_fixture_tests.rs"]
mod real_source_fixture_tests;
