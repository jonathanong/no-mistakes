use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::dependencies::extract::{ExportedBinding, ImportedBindingKind};
use crate::codebase::dependencies::graph::{
    DepGraph, EdgeKind, GraphBuildPlan, NodeId, TsFactLookup,
};
use crate::config::v2::NoMistakesConfig;
use std::path::{Path, PathBuf};

pub(crate) fn graph_plan(config: &NoMistakesConfig) -> Option<GraphBuildPlan> {
    super::enabled(config).then_some(GraphBuildPlan {
        calls: true,
        imports: true,
        ..GraphBuildPlan::default()
    })
}

/// A uniquely resolved local export imported by `importer` under `local`.
pub(crate) struct UniqueExport {
    pub producer: PathBuf,
    pub exported: String,
}

pub(crate) fn unique_export(
    graph: &DepGraph,
    facts: &CheckFactMap,
    importer: &Path,
    local: &str,
) -> Option<UniqueExport> {
    let ts = facts.get_ts_facts(importer)?;
    if ts.parse_error.is_some() || ts.operational_error.is_some() {
        return None;
    }
    let binding = ts
        .imported_bindings
        .iter()
        .find(|binding| !binding.is_type_only && binding.local == local)?;
    if binding.kind == ImportedBindingKind::Namespace {
        return None;
    }
    let imported = binding.imported.clone();
    let node = NodeId::file(importer);
    let mut producers = Vec::new();
    for (target, kind) in graph.dependencies_of_node(&node)? {
        if !matches!(kind, EdgeKind::Import | EdgeKind::Require) {
            continue;
        }
        let Some(file) = target.as_file() else {
            continue;
        };
        if local_export(facts, file, &imported) {
            producers.push(file.to_path_buf());
        }
    }
    producers.sort();
    producers.dedup();
    let [producer] = producers.as_slice() else {
        return None;
    };
    Some(UniqueExport {
        producer: producer.clone(),
        exported: imported,
    })
}

fn local_export(facts: &CheckFactMap, file: &Path, imported: &str) -> bool {
    let Some(ts) = facts.get_ts_facts(file) else {
        return false;
    };
    if ts.parse_error.is_some() || ts.operational_error.is_some() {
        return false;
    }
    ts.exported_bindings
        .iter()
        .any(|binding| local_exported_name(binding, imported))
}

fn local_exported_name(binding: &ExportedBinding, imported: &str) -> bool {
    binding.specifier.is_none() && binding.exported == imported
}

#[cfg(test)]
mod tests;
