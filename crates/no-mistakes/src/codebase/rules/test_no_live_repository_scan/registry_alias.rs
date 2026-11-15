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
    let mut found = None;
    for (file, local) in homes(graph, facts, site, object) {
        let Some(ts) = facts.get_ts_facts(&file) else {
            continue;
        };
        let Some(name) = terminal_alias(ts, &format!("{local}.{key}")) else {
            continue;
        };
        let Some(node) = callable(graph, &file, ts, &name) else {
            continue;
        };
        if found.is_some() {
            return None;
        }
        found = Some(node);
    }
    found
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
            let Some(ts) = facts.get_ts_facts(&file) else {
                return false;
            };
            let prefix = format!("{local}.");
            ts.callable_aliases.iter().any(|alias| {
                alias.scope_id.is_none()
                    && alias.invalidated_at.is_none()
                    && alias.local.starts_with(&prefix)
                    && terminal_alias(ts, &alias.local)
                        .and_then(|name| callable(graph, &file, ts, &name))
                        .is_some_and(|node| reaches_scan(graph, enumerators, &node))
            })
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
    let mut homes = Vec::new();
    let Some(caller) = facts.get_ts_facts(&site.file) else {
        return homes;
    };
    let prefix = format!("{object}.");
    if caller.callable_aliases.iter().any(|alias| {
        alias.scope_id.is_none()
            && alias.invalidated_at.is_none()
            && alias.local.starts_with(&prefix)
    }) {
        homes.push((site.file.clone(), object.to_string()));
    }
    let Some(imported) = caller.imported_bindings.iter().find(|binding| {
        binding.local == object
            && !binding.is_type_only
            && binding.kind == ImportedBindingKind::Named
    }) else {
        return homes;
    };
    for file in direct_imports(graph, &site.file) {
        let Some(home) = facts.get_ts_facts(&file) else {
            continue;
        };
        let Some(local) = home
            .exported_bindings
            .iter()
            .find(|export| export.exported == imported.imported && export.specifier.is_none())
            .map(|export| export.local.clone())
        else {
            continue;
        };
        homes.push((file, local));
    }
    homes
}

fn terminal_alias(ts: &TsFileFacts, local: &str) -> Option<String> {
    let mut name = local.to_string();
    for _ in 0..8 {
        let alias = ts.callable_aliases.iter().find(|alias| {
            alias.scope_id.is_none() && alias.invalidated_at.is_none() && alias.local == name
        })?;
        if !alias.target.contains('.')
            && ts
                .callable_scope_ids
                .iter()
                .any(|(_, scope)| scope == &alias.target)
        {
            return Some(alias.target.clone());
        }
        if alias.target == name {
            return None;
        }
        name = alias.target.clone();
    }
    None
}

fn callable(graph: &DepGraph, file: &Path, ts: &TsFileFacts, name: &str) -> Option<NodeId> {
    let id = ts
        .callable_scope_ids
        .iter()
        .find(|(_, scope)| scope == name)
        .map(|(id, _)| *id)?;
    Some(
        graph
            .resolved_call_sites()
            .iter()
            .find_map(|site| {
                (site.file == file && site.caller_id == Some(id)).then(|| site_source_node(site))
            })
            .unwrap_or_else(|| NodeId::callable(file, name, id)),
    )
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
