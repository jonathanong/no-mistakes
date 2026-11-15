//! Which function a registry member names, and whether that function scans.
use super::super::super::forbidden_calls::site_source_node;
use super::super::config::Enumerator;
use super::super::enumerators::scan_of;
use crate::codebase::dependencies::extract::ImportedBindingKind;
use crate::codebase::dependencies::graph::{
    CallTraversal, DepGraph, EdgeKind, NodeId, ResolvedCallSite, TsFactLookup,
};
use crate::codebase::ts_source::facts::TsFileFacts;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub(super) fn function_node(
    graph: &DepGraph,
    facts: &dyn TsFactLookup,
    site: &ResolvedCallSite,
    object: &str,
    key: &str,
) -> Option<NodeId> {
    homes(graph, facts, site, object)
        .into_iter()
        .find_map(|(file, local)| {
            let ts = facts.get_ts_facts(&file)?;
            let name = terminal_alias(ts, &format!("{local}.{key}"))?;
            callable(graph, &file, ts, &name)
        })
}

pub(super) fn registry_scans(
    graph: &DepGraph,
    facts: &dyn TsFactLookup,
    site: &ResolvedCallSite,
    object: &str,
    enumerators: &[Enumerator],
) -> bool {
    homes(graph, facts, site, object)
        .into_iter()
        .any(|(file, local)| {
            facts
                .get_ts_facts(&file)
                .is_some_and(|ts| scans_member(graph, ts, &file, &local, enumerators))
        })
}

fn scans_member(
    graph: &DepGraph,
    ts: &TsFileFacts,
    file: &Path,
    local: &str,
    enumerators: &[Enumerator],
) -> bool {
    let prefix = format!("{local}.");
    ts.callable_aliases.iter().any(|alias| {
        alias.scope_id.is_none()
            && alias.invalidated_at.is_none()
            && alias.local.starts_with(&prefix)
            && terminal_alias(ts, &alias.local)
                .and_then(|name| callable(graph, file, ts, &name))
                .is_some_and(|node| reaches_scan(graph, enumerators, &node))
    })
}

fn reaches_scan(graph: &DepGraph, enumerators: &[Enumerator], node: &NodeId) -> bool {
    let sources =
        graph.call_source_nodes(std::slice::from_ref(node), CallTraversal::Transitive, None);
    graph.resolved_call_sites().iter().any(|site| {
        sources.contains(&site_source_node(site)) && scan_of(site, enumerators).is_some()
    })
}

/// The module that defines `object`, and the local binding name of that object.
fn homes(
    graph: &DepGraph,
    facts: &dyn TsFactLookup,
    site: &ResolvedCallSite,
    object: &str,
) -> Vec<(PathBuf, String)> {
    facts
        .get_ts_facts(&site.file)
        .map(|caller| {
            let mut found = Vec::new();
            if local_registry(caller, object) {
                found.push((site.file.clone(), object.to_string()));
            }
            if let Some(imported) = imported_name(caller, object) {
                found.extend(
                    direct_imports(graph, &site.file)
                        .into_iter()
                        .filter_map(|file| export_local(facts, &file, &imported)),
                );
            }
            found
        })
        .unwrap_or_default()
}

fn local_registry(caller: &TsFileFacts, object: &str) -> bool {
    let prefix = format!("{object}.");
    caller.callable_aliases.iter().any(|alias| {
        alias.scope_id.is_none()
            && alias.invalidated_at.is_none()
            && alias.local.starts_with(&prefix)
    })
}

fn imported_name(caller: &TsFileFacts, object: &str) -> Option<String> {
    caller
        .imported_bindings
        .iter()
        .find(|binding| {
            binding.local == object
                && !binding.is_type_only
                && binding.kind == ImportedBindingKind::Named
        })
        .map(|binding| binding.imported.clone())
}

fn export_local(
    facts: &dyn TsFactLookup,
    file: &Path,
    imported: &str,
) -> Option<(PathBuf, String)> {
    let home = facts.get_ts_facts(file)?;
    home.exported_bindings.iter().find_map(|export| {
        (export.exported == imported && export.specifier.is_none())
            .then(|| (file.to_path_buf(), export.local.clone()))
    })
}

fn terminal_alias(ts: &TsFileFacts, local: &str) -> Option<String> {
    let alias = ts.callable_aliases.iter().find(|alias| {
        alias.scope_id.is_none() && alias.invalidated_at.is_none() && alias.local == local
    })?;
    (!alias.target.contains('.')
        && ts
            .callable_scope_ids
            .iter()
            .any(|(_, scope)| scope == &alias.target))
    .then(|| alias.target.clone())
}

fn callable(graph: &DepGraph, file: &Path, ts: &TsFileFacts, name: &str) -> Option<NodeId> {
    let id = ts
        .callable_scope_ids
        .iter()
        .find(|(_, scope)| scope == name)
        .map(|(id, _)| *id)?;
    graph.resolved_call_sites().iter().find_map(|site| {
        (site.file == file && site.caller_id == Some(id)).then(|| site_source_node(site))
    })
}

fn direct_imports(graph: &DepGraph, file: &Path) -> Vec<PathBuf> {
    let kinds = [EdgeKind::Import, EdgeKind::WorkspaceImport]
        .into_iter()
        .collect::<HashSet<_>>();
    graph
        .deps_of(&[NodeId::file(file)], Some(1), Some(&kinds))
        .into_iter()
        .filter_map(|entry| entry.node.as_file().map(Path::to_path_buf))
        .collect()
}
