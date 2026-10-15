#[path = "receiver_evaluation_order_cases.rs"]
mod cases;
#[path = "receiver_enclosing_init_cases.rs"]
mod enclosing_init_cases;
#[path = "receiver_parameter_order_cases.rs"]
mod parameter_cases;

use super::*;

#[test]
fn imported_receiver_reads_follow_assignment_and_invocation_order() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture(
        "value-flow-receiver-evaluation-order",
    ));
    let config = TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    let graph = DepGraph::build_with_plan(
        &root,
        &config,
        GraphBuildPlan {
            calls: true,
            ..Default::default()
        },
    )
    .unwrap();
    for (file, caller, callee, export) in cases::cases()
        .iter()
        .chain(parameter_cases::cases())
        .chain(enclosing_init_cases::cases())
        .copied()
    {
        let sites = graph.call_sites_in_file(&root.join(format!("src/{file}.mts")));
        let line = match callee {
            "client().get" => Some(4),
            "clientAfter().get" => Some(8),
            _ => None,
        };
        let recorded = if line.is_some() {
            "<unknown>.get"
        } else {
            callee
        };
        let site = sites
            .iter()
            .find(|site| {
                site.caller.as_deref() == caller
                    && site.source_callee == recorded
                    && line.is_none_or(|line| site.line == line)
            })
            .unwrap_or_else(|| panic!("{file} {caller:?} {callee}: {sites:?}"));
        if matches!(
            file,
            "direct-inline-arguments" | "repeated-binding-acquisitions"
        ) {
            assert_eq!(
                site.invocation,
                crate::codebase::dependencies::extract::InvocationKind::Call
            );
        }
        assert_eq!(
            match &site.target {
                ResolvedCallTarget::ModuleExport { export_path, .. } => Some(export_path.as_str()),
                _ => None,
            },
            export,
            "{file} {caller:?} {callee}: {site:?}"
        );
    }
    // The module copy predates every invocation, including repeated writes.
    let sites = graph.call_sites_in_file(&root.join("src/repeated-parameter-phase.mts"));
    let snapshot = sites
        .iter()
        .find(|site| {
            site.caller.as_deref() == Some("runSaved/repeatedSaved") && site.source_callee == "fn"
        })
        .expect("module copied-method site");
    assert!(
        matches!(&snapshot.target, ResolvedCallTarget::ModuleExport { export_path, .. }
        if export_path == "savedPhase.get"),
        "{snapshot:?}"
    );
    let direct = graph.call_sites_in_file(&root.join("src/direct-inline-arguments.mts"));
    // These are real canonical incoming calls, not synthetic membership rows.
    for (caller, scope) in [
        (None, "runDirect"),
        (Some("runDirect"), "runDirect/takeLive"),
    ] {
        let incoming = direct.iter().filter(|site| {
            site.caller.as_deref() == caller
                && matches!(&site.target, ResolvedCallTarget::RepositoryFunction { scope: target, .. } if target == scope)
        }).collect::<Vec<_>>();
        assert_eq!(incoming.len(), 1, "{scope}: {direct:?}");
        assert_eq!(
            incoming[0].invocation,
            crate::codebase::dependencies::extract::InvocationKind::Call
        );
        assert!(incoming[0].target_node.is_some(), "{scope}");
    }
    let repeated = graph.call_sites_in_file(&root.join("src/repeated-binding-acquisitions.mts"));
    for (caller, scope) in [
        (None, "runRepeated"),
        (Some("runRepeated"), "runRepeated/takeRepeated"),
    ] {
        let incoming = repeated.iter().filter(|site| site.caller.as_deref() == caller && matches!(&site.target, ResolvedCallTarget::RepositoryFunction { scope: target, .. } if target == scope)).collect::<Vec<_>>();
        assert_eq!(incoming.len(), 1, "{scope}: {repeated:?}");
        assert_eq!(
            incoming[0].invocation,
            crate::codebase::dependencies::extract::InvocationKind::Call
        );
        assert!(incoming[0].target_node.is_some(), "{scope}");
    }
}
