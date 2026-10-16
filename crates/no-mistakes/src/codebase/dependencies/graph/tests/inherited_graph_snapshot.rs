// Test-only canonical projection extracted from frozen ARM graph-probe.rs.
use super::*;
use std::path::Path;

fn quote(value: &str) -> String {
    let mut result = String::from("\"");
    for character in value.chars() {
        match character {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            character if character.is_control() => {
                result.push_str(&format!("\\u{:04x}", character as u32))
            }
            character => result.push(character),
        }
    }
    result.push('"');
    result
}
fn optional(value: Option<String>) -> String {
    value.map_or_else(|| "null".into(), |value| quote(&value))
}
fn path_key(path: &Path, root: &Path) -> String {
    quote(&path.strip_prefix(root).unwrap_or(path).to_string_lossy())
}
fn node_key(node: &NodeId, root: &Path) -> String {
    match node {
        NodeId::File(file) => format!("[\"file\",{}]", path_key(file.as_ref(), root)),
        NodeId::Symbol {
            file,
            symbol,
            callable_id,
        } => format!(
            "[\"symbol\",{},{},{}]",
            path_key(file.as_ref(), root),
            quote(symbol.as_ref()),
            quote(&format!("{callable_id:?}"))
        ),
        NodeId::Module(module) => format!("[\"module\",{}]", quote(module.as_ref())),
        NodeId::QueueJob { queue_file, job } => format!(
            "[\"queue-job\",{},{}]",
            path_key(queue_file.as_ref(), root),
            quote(job.as_ref())
        ),
        NodeId::WorkflowJob { workflow_file, job } => format!(
            "[\"workflow-job\",{},{}]",
            path_key(workflow_file.as_ref(), root),
            quote(job.as_ref())
        ),
        NodeId::WorkflowStep {
            workflow_file,
            job,
            step,
        } => format!(
            "[\"workflow-step\",{},{},{}]",
            path_key(workflow_file.as_ref(), root),
            quote(job.as_ref()),
            step
        ),
        NodeId::TrpcProcedure {
            router_file,
            procedure,
        } => format!(
            "[\"trpc-procedure\",{},{}]",
            path_key(router_file.as_ref(), root),
            quote(procedure.as_ref())
        ),
    }
}
fn leaf(
    kind: &str,
    module: &str,
    name: &str,
    repository: Option<(&std::path::Path, &str)>,
    id: String,
    node: Option<&NodeId>,
    root: &Path,
) -> String {
    format!("{{\"kind\":{},\"module\":{},\"name\":{},\"repository\":{},\"callable_id\":{},\"node\":{}}}",
        quote(kind), quote(module), quote(name), optional(repository.map(|(file, scope)| format!("{}#{scope}", file.display()))),
        quote(&id), optional(node.map(|node| node_key(node, root))))
}
fn targets(site: &ResolvedCallSite, root: &Path) -> (Vec<String>, bool) {
    use crate::codebase::dependencies::graph::KnownCallTargetRef;
    let rows = site
        .targets()
        .map(|candidate| match candidate.target {
            KnownCallTargetRef::Global { name } => leaf(
                "global",
                "",
                name,
                None,
                "None".into(),
                candidate.node,
                root,
            ),
            KnownCallTargetRef::ModuleExport {
                specifier,
                export_path,
                repository_target,
                callable_id,
            } => leaf(
                "export",
                specifier,
                export_path,
                repository_target,
                format!("{callable_id:?}"),
                candidate.node,
                root,
            ),
            KnownCallTargetRef::RepositoryFunction { file, scope } => leaf(
                "repository",
                "",
                scope,
                Some((file, scope)),
                "None".into(),
                candidate.node,
                root,
            ),
        })
        .collect();
    (rows, site.has_unknown_target())
}
pub(super) fn render(graph: &DepGraph, root: &Path) -> serde_json::Value {
    let sites = graph.resolved_call_sites();
    let nodes: Vec<_> = graph.all_files().cloned().collect();
    let mut node_rows: Vec<_> = nodes
        .iter()
        .map(|node| quote(&node_key(node, root)))
        .collect();
    node_rows.sort();
    let mut edges = Vec::new();
    let mut incoming = Vec::new();
    for node in nodes {
        for (target, kind) in graph.dependencies_of_node(&node).into_iter().flatten() {
            edges.push(format!(
                "[{}, {}, {}]",
                quote(&node_key(&node, root)),
                quote(&node_key(target, root)),
                quote(&format!("{kind:?}"))
            ));
        }
        for (source, kind) in graph.dependents_of_node(&node).into_iter().flatten() {
            incoming.push(format!(
                "[{}, {}, {}]",
                quote(&node_key(source, root)),
                quote(&node_key(&node, root)),
                quote(&format!("{kind:?}"))
            ));
        }
    }
    edges.sort();
    edges.dedup();
    incoming.sort();
    incoming.dedup();
    let mut rows = Vec::new();
    for site in sites {
        let (mut known, unknown) = targets(site, root);
        known.sort();
        known.dedup();
        rows.push(format!("{{\"file\":{},\"offset\":{},\"line\":{},\"caller\":{},\"caller_id\":{},\"invocation\":{},\"callee\":{},\"known\":[{}],\"unknown\":{},\"returned_from\":{},\"typed_from\":{},\"returned_members\":{}}}",
                quote(&site.file.strip_prefix(root).unwrap_or(&site.file).to_string_lossy()), site.offset, site.line,
                optional(site.caller.clone()), quote(&format!("{:?}", site.caller_id)), quote(&format!("{:?}", site.invocation)),
                quote(&site.source_callee), known.join(","), unknown, quote(&format!("{:?}", site.returned_from)),
                quote(&format!("{:?}", site.typed_from)), quote(&format!("{:?}", site.returned_member_from))));
    }
    rows.sort();
    let semantic = format!(
        "{{\"sites\":[{}],\"edges\":[{}],\"nodes\":[{}],\"importer_view\":[{}]}}",
        rows.join(","),
        edges.join(","),
        node_rows.join(","),
        incoming.join(",")
    );
    serde_json::from_str(&semantic).unwrap()
}

/// Normalize only a string's declared physical fixture-root prefix, retaining all fields.
pub(super) fn normalize_fixture_root(value: &mut serde_json::Value, root: &Path) {
    match value {
        serde_json::Value::String(text) => {
            if let Some(suffix) = text.strip_prefix(root.to_str().unwrap()) {
                assert!(suffix.is_empty() || suffix.starts_with('/') || suffix.starts_with('#'));
                *text = format!("<ROOT>{suffix}");
            }
        }
        serde_json::Value::Array(values) => {
            for value in values {
                normalize_fixture_root(value, root);
            }
        }
        serde_json::Value::Object(values) => {
            for value in values.values_mut() {
                normalize_fixture_root(value, root);
            }
        }
        _ => {}
    }
}
