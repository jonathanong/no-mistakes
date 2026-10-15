//! Verification-only. Compile the same source with --cfg alternatives_api for
//! the candidate bridge. Freeze each artifact against its exact matched source arm.
#[cfg(not(alternatives_api))]
use no_mistakes::codebase::dependencies::graph::ResolvedCallTarget;
use no_mistakes::codebase::dependencies::{
    graph::{DepGraph, GraphBuildPlan, NodeId, ResolvedCallSite},
    TsConfig,
};
use no_mistakes::diagnostics::{with_observer, InvocationObserver};
use std::{hint::black_box, path::PathBuf};

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
fn leaf(
    kind: &str,
    module: &str,
    name: &str,
    repository: Option<(&std::path::Path, &str)>,
    id: String,
    node: Option<&NodeId>,
) -> String {
    format!("{{\"kind\":{},\"module\":{},\"name\":{},\"repository\":{},\"callable_id\":{},\"node\":{}}}",
        quote(kind), quote(module), quote(name), optional(repository.map(|(file, scope)| format!("{}#{scope}", file.display()))),
        quote(&id), optional(node.map(|node| format!("{node:?}"))))
}
#[cfg(not(alternatives_api))]
fn targets(site: &ResolvedCallSite) -> (Vec<String>, bool) {
    let row = match &site.target {
        ResolvedCallTarget::Global { name } => leaf(
            "global",
            "",
            name,
            None,
            "None".into(),
            site.target_node.as_ref(),
        ),
        ResolvedCallTarget::ModuleExport {
            specifier,
            export_path,
            repository_target,
            callable_id,
        } => leaf(
            "export",
            specifier,
            export_path,
            repository_target
                .as_ref()
                .map(|(file, scope)| (file.as_path(), scope.as_str())),
            format!("{callable_id:?}"),
            site.target_node.as_ref(),
        ),
        ResolvedCallTarget::RepositoryFunction { file, scope } => leaf(
            "repository",
            "",
            scope,
            Some((file.as_path(), scope)),
            "None".into(),
            site.target_node.as_ref(),
        ),
        ResolvedCallTarget::Unknown => return (Vec::new(), true),
    };
    (vec![row], false)
}
#[cfg(alternatives_api)]
fn targets(site: &ResolvedCallSite) -> (Vec<String>, bool) {
    use no_mistakes::codebase::dependencies::graph::KnownCallTargetRef;
    let rows = site
        .targets()
        .map(|candidate| match candidate.target {
            KnownCallTargetRef::Global { name } => {
                leaf("global", "", name, None, "None".into(), candidate.node)
            }
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
            ),
            KnownCallTargetRef::RepositoryFunction { file, scope } => leaf(
                "repository",
                "",
                scope,
                Some((file, scope)),
                "None".into(),
                candidate.node,
            ),
        })
        .collect();
    (rows, site.has_unknown_target())
}
fn main() {
    let args: Vec<_> = std::env::args().collect();
    assert_eq!(args.len(), 5, "ROOT ITERATIONS off|work calls|no-calls");
    let root = PathBuf::from(&args[1]).canonicalize().unwrap();
    let iterations: usize = args[2].parse().unwrap();
    let calls = match args[4].as_str() {
        "calls" => true,
        "no-calls" => false,
        _ => panic!("calls mode"),
    };
    let work_enabled = match args[3].as_str() {
        "off" => false,
        "work" => true,
        _ => panic!("mode"),
    };
    assert!(iterations > 0);
    let config = TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    let mut prior = None;
    let mut work_rows = Vec::new();
    for _ in 0..iterations {
        let observer = work_enabled.then(|| InvocationObserver::new(true));
        let graph = with_observer(observer.clone(), || {
            DepGraph::build_with_plan(
                &root,
                &config,
                GraphBuildPlan {
                    imports: true,
                    calls,
                    ..Default::default()
                },
            )
            .unwrap()
        });
        let sites = graph.resolved_call_sites();
        let nodes: Vec<_> = graph.all_files().cloned().collect();
        let mut node_rows: Vec<_> = nodes
            .iter()
            .map(|node| quote(&format!("{node:?}")))
            .collect();
        node_rows.sort();
        let mut edges = Vec::new();
        let mut incoming = Vec::new();
        for node in nodes {
            for (target, kind) in graph.dependencies_of_node(&node).into_iter().flatten() {
                edges.push(format!(
                    "[{}, {}, {}]",
                    quote(&format!("{node:?}")),
                    quote(&format!("{target:?}")),
                    quote(&format!("{kind:?}"))
                ));
            }
            for (source, kind) in graph.dependents_of_node(&node).into_iter().flatten() {
                incoming.push(format!(
                    "[{}, {}, {}]",
                    quote(&format!("{source:?}")),
                    quote(&format!("{node:?}")),
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
            let (mut known, unknown) = targets(site);
            known.sort();
            known.dedup();
            rows.push(format!("{{\"file\":{},\"offset\":{},\"line\":{},\"caller\":{},\"caller_id\":{},\"invocation\":{},\"callee\":{},\"known\":[{}],\"unknown\":{},\"returned_from\":{},\"typed_from\":{},\"returned_members\":{}}}",
                quote(&site.file.strip_prefix(&root).unwrap_or(&site.file).to_string_lossy()), site.offset, site.line,
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
        if let Some(previous) = &prior {
            assert_eq!(previous, &semantic, "fresh requests must agree");
        }
        prior = Some(semantic);
        if let Some(observer) = observer {
            let work = observer
                .snapshot()
                .work
                .into_iter()
                .map(|(name, count)| format!("{}:{count}", quote(name)))
                .collect::<Vec<_>>()
                .join(",");
            work_rows.push(format!("{{{work}}}"));
        }
        black_box(graph);
    }
    // Actual incoming ParameterInputs are private; do not infer their parity
    // from this public edge view. The separate internal ledger is mandatory.
    println!(
        "{{\"semantic\":{},\"request_work\":[{}],\"actual_incoming\":null}}",
        prior.unwrap(),
        work_rows.join(",")
    );
}
