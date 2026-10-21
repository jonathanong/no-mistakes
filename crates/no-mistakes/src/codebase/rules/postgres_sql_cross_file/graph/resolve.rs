pub(super) use crate::codebase::check_facts::CheckFactMap;
pub(super) use crate::codebase::dependencies::extract::{ExportedBinding, ImportedBindingKind};
pub(super) use crate::codebase::dependencies::graph::{DepGraph, EdgeKind, NodeId, TsFactLookup};
pub(super) use std::path::{Path, PathBuf};

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
    let Some(binding) = imported(facts, importer, local) else {
        return super::routes::registry_route(graph, facts, importer, local, &[]);
    };
    if binding.kind == ImportedBindingKind::Namespace {
        return None;
    }
    unique_from_imports(graph, facts, importer, &binding.imported)
}

pub(crate) fn unique_namespace_member(
    graph: &DepGraph,
    facts: &CheckFactMap,
    importer: &Path,
    local: &str,
    member: &str,
) -> Option<UniqueExport> {
    let binding = imported(facts, importer, local)?;
    if binding.kind != ImportedBindingKind::Namespace {
        return None;
    }
    unique_from_imports(graph, facts, importer, member)
}

pub(super) fn imported<'a>(
    facts: &'a CheckFactMap,
    importer: &Path,
    local: &str,
) -> Option<&'a crate::codebase::dependencies::extract::ImportedBinding> {
    let ts = facts.get_ts_facts(importer)?;
    if ts.parse_error.is_some() || ts.operational_error.is_some() {
        return None;
    }
    ts.imported_bindings
        .iter()
        .find(|binding| !binding.is_type_only && binding.local == local)
}

fn unique_from_imports(
    graph: &DepGraph,
    facts: &CheckFactMap,
    importer: &Path,
    exported: &str,
) -> Option<UniqueExport> {
    let mut producers = Vec::new();
    let mut visited = Vec::new();
    for target in import_files(graph, importer) {
        producers.extend(defining_modules(
            graph,
            facts,
            &target,
            exported,
            &mut visited,
        ));
    }
    producers.sort();
    producers.dedup();
    let [producer] = producers.as_slice() else {
        return None;
    };
    Some(UniqueExport {
        producer: producer.clone(),
        exported: exported.to_string(),
    })
}

fn defining_modules(
    graph: &DepGraph,
    facts: &CheckFactMap,
    file: &Path,
    exported: &str,
    visited: &mut Vec<(PathBuf, String)>,
) -> Vec<PathBuf> {
    let key = (file.to_path_buf(), exported.to_string());
    if visited.contains(&key) {
        return Vec::new();
    }
    visited.push(key);
    let Some(ts) = facts.get_ts_facts(file) else {
        return Vec::new();
    };
    if ts.parse_error.is_some() || ts.operational_error.is_some() {
        return Vec::new();
    }
    if local_export(&ts.exported_bindings, exported) {
        return vec![file.to_path_buf()];
    }
    if let Some(binding) = ts
        .exported_bindings
        .iter()
        .find(|binding| binding.exported == exported && binding.specifier.is_some())
    {
        return unique_import_definitions(graph, facts, file, &binding.local, visited);
    }
    if ts.star_reexport_specifiers.is_empty() {
        return Vec::new();
    }
    unique_import_definitions(graph, facts, file, exported, visited)
}

fn unique_import_definitions(
    graph: &DepGraph,
    facts: &CheckFactMap,
    file: &Path,
    exported: &str,
    visited: &mut Vec<(PathBuf, String)>,
) -> Vec<PathBuf> {
    let mut producers = Vec::new();
    for target in import_files(graph, file) {
        producers.extend(defining_modules(graph, facts, &target, exported, visited));
    }
    producers.sort();
    producers.dedup();
    producers
}

fn import_files(graph: &DepGraph, importer: &Path) -> Vec<PathBuf> {
    let Some(neighbors) = graph.dependencies_of_node(&NodeId::file(importer)) else {
        return Vec::new();
    };
    neighbors
        .iter()
        .filter_map(|(target, kind)| {
            matches!(kind, EdgeKind::Import | EdgeKind::Require)
                .then(|| target.as_file().map(Path::to_path_buf))
                .flatten()
        })
        .collect()
}

fn local_export(bindings: &[ExportedBinding], imported: &str) -> bool {
    bindings
        .iter()
        .any(|binding| binding.specifier.is_none() && binding.exported == imported)
}
