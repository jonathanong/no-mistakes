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
        assert_eq!(
            match &site.target {
                ResolvedCallTarget::ModuleExport { export_path, .. } => Some(export_path.as_str()),
                _ => None,
            },
            export,
            "{file} {caller:?} {callee}: {site:?}"
        );
    }
    // #2033 is a current-parent precision gap, not a supported Unknown policy:
    // exact4ddb and e1 report Unknown; desired JS target is savedPhase.get.
    // Keep this separate from supported positives while reusing the same graph.
    let sites = graph.call_sites_in_file(&root.join("src/repeated-parameter-phase.mts"));
    let snapshot = sites
        .iter()
        .find(|site| {
            site.caller.as_deref() == Some("runSaved/repeatedSaved") && site.source_callee == "fn"
        })
        .expect("#2033 copied-method precision-gap site");
    assert!(
        matches!(&snapshot.target, ResolvedCallTarget::Unknown),
        "#2033 current-parent gap; desired savedPhase.get: {snapshot:?}"
    );
}
