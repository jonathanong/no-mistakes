use super::*;

fn fixture() -> std::path::PathBuf {
    root().with_file_name("call-reachability-fixture")
}

#[test]
fn closed_stage_methods_and_invoked_thunks_reach_only_their_proven_queries() {
    let findings = crate::codebase::rules::run_check(&fixture(), None, None).unwrap();
    let occurrences = findings
        .iter()
        .map(|finding| (finding.line, finding.import.as_deref().unwrap()))
        .collect::<Vec<_>>();
    assert_eq!(
        occurrences,
        [
            (11, "stage.run"),
            (15, "both"),
            (28, "objectStage.run"),
            (59, "stage.run"),
            (67, "stage.run"),
            (76, "stage.run"),
            (89, "stage.run")
        ],
        "{findings:#?}"
    );
    let stage = findings.iter().find(|finding| finding.line == 11).unwrap();
    assert!(stage.message.contains("q"), "{stage:?}");
    assert!(findings
        .iter()
        .find(|finding| finding.line == 15)
        .unwrap()
        .message
        .contains("both →"));
}

#[test]
fn stage_methods_are_shared_canonical_call_targets() {
    use crate::codebase::dependencies::graph::{DepGraph, GraphBuildPlan};
    let root = crate::codebase::ts_resolver::normalize_path(&fixture());
    let config = crate::codebase::ts_resolver::TsConfig {
        dir: root.clone(),
        paths_dir: root.clone(),
        ..Default::default()
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
    let stage = graph
        .resolved_call_sites()
        .iter()
        .find(|site| site.file == root.join("src/entry.mts") && site.line == 11)
        .unwrap();
    assert!(!stage.has_unknown_target(), "{stage:?}");
    assert_eq!(stage.target_nodes().count(), 2, "{stage:?}");
    for line in [33, 37, 41, 47, 55, 63, 64, 72, 80, 84] {
        let site = graph
            .resolved_call_sites()
            .iter()
            .find(|site| {
                site.file == stage.file && site.line == line && site.source_callee == "stage.run"
            })
            .unwrap();
        assert!(site.target_nodes().next().is_none(), "{site:?}");
    }
}

#[test]
fn imported_multi_method_stages_forward_typed_query_parameters() {
    use crate::codebase::dependencies::graph::{DepGraph, GraphBuildPlan};
    let root = crate::codebase::ts_resolver::normalize_path(
        &root().with_file_name("imported-stages-fixture"),
    );
    let config = crate::codebase::ts_resolver::TsConfig {
        dir: root.clone(),
        paths_dir: root.clone(),
        ..Default::default()
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
    for site in graph.resolved_call_sites().iter().filter(|site| {
        site.file == root.join("src/controls.mts") && site.source_callee == "stage.run"
    }) {
        assert!(site.target_nodes().next().is_none(), "{site:?}");
    }
    for line in [5, 6] {
        let site = graph
            .resolved_call_sites()
            .iter()
            .find(|site| site.file == root.join("src/caller.mts") && site.line == line)
            .unwrap();
        assert!(!site.has_unknown_target(), "{site:?}");
        assert_eq!(site.target_nodes().count(), 2, "{site:?}");
        assert!(
            site.target_nodes()
                .all(|node| matches!(node, NodeId::Symbol { file, .. }
            if file.as_ref() == root.join("src/stages.mts"))),
            "{site:?}"
        );
    }
    let findings = crate::codebase::rules::run_check(&root, None, None).unwrap();
    assert_eq!(
        findings
            .iter()
            .map(|finding| (finding.line, finding.import.as_deref()))
            .collect::<Vec<_>>(),
        [(5, Some("stage.run")), (6, Some("stage.check"))],
        "{findings:#?}"
    );
    assert!(findings
        .iter()
        .all(|finding| finding.message.contains("transaction client only")));
}

#[test]
fn exported_stage_arrays_retain_real_escapes_separately_from_export_declarations() {
    let root = root().with_file_name("imported-stages-fixture");
    let stages = root.join("src/stages.mts");
    let controls = root.join("src/control-stages.mts");
    let importer = root.join("src/no-loop-importer.mts");
    let facts = crate::codebase::ts_source::facts::collect_ts_facts(
        &[stages.clone(), controls.clone(), importer.clone()],
        crate::codebase::dependencies::graph::GraphBuildPlan {
            calls: true,
            ..Default::default()
        }
        .ts_fact_plan(),
    );
    let clean = &facts.get(&stages).unwrap().value_flow.for_of;
    assert!(clean.exported[&0].contains("STAGES"));
    assert!(!clean.escaped.as_ref().is_some_and(|scopes| {
        scopes
            .get(&0)
            .is_some_and(|names| names.contains_key("STAGES"))
    }));
    for (file, names) in [
        (&controls, ["MUTATED", "ESCAPED"]),
        (&importer, ["SIBLING_MUTATED", "SIBLING_ESCAPED"]),
    ] {
        let escaped = facts
            .get(file)
            .unwrap()
            .value_flow
            .for_of
            .escaped
            .as_ref()
            .unwrap();
        for name in names {
            assert!(escaped[&0].contains_key(name), "{file:?}: {name}");
        }
    }
}

#[test]
fn namespace_members_stars_cycles_and_unresolved_stars_prevent_imported_stage_proofs() {
    use crate::codebase::dependencies::graph::{DepGraph, GraphBuildPlan};
    for name in [
        "namespace-member-stages-fixture",
        "namespace-cycle-stages-fixture",
        "namespace-unknown-stages-fixture",
    ] {
        let root = crate::codebase::ts_resolver::normalize_path(&root().with_file_name(name));
        let findings = crate::codebase::rules::run_check(&root, None, None).unwrap();
        assert!(findings.is_empty(), "{name}: {findings:?}");
        let config = crate::codebase::ts_resolver::TsConfig {
            dir: root.clone(),
            paths_dir: root.clone(),
            ..Default::default()
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
        let site = graph
            .resolved_call_sites()
            .iter()
            .find(|site| {
                site.file == root.join("src/caller.mts") && site.source_callee == "stage.run"
            })
            .unwrap();
        assert!(site.target_nodes().next().is_none(), "{name}: {site:?}");
    }
}

#[test]
fn workspace_stage_results_keep_query_reachability_across_terminal_checkpoint_guards() {
    let root = root().with_file_name("guarded-workspace-stages-fixture");
    for config in [".no-mistakes.yml", "typed.yml"] {
        let findings =
            crate::codebase::rules::run_check(&root, Some(&root.join(config)), None).unwrap();
        assert_eq!(
            findings
                .iter()
                .map(|finding| (
                    finding.file.as_str(),
                    finding.line,
                    finding.import.as_deref()
                ))
                .collect::<Vec<_>>(),
            [
                ("src/caller.mts", 7, Some("stage.deletion")),
                ("src/caller.mts", 12, Some("stage.remaining"))
            ],
            "{config}: {findings:#?}"
        );
    }
}

#[test]
fn terminal_guards_do_not_prove_every_imported_scenario_row_executes() {
    use crate::codebase::dependencies::graph::{DepGraph, GraphBuildPlan};
    let root = crate::codebase::ts_resolver::normalize_path(
        &root().with_file_name("guarded-workspace-stages-fixture"),
    );
    let config = crate::codebase::ts_resolver::TsConfig {
        dir: root.clone(),
        paths_dir: root.clone(),
        ..Default::default()
    };
    let graph = DepGraph::build_with_plan(
        &root,
        &config,
        GraphBuildPlan {
            calls: true,
            assertion_owners: true,
            ..Default::default()
        },
    )
    .unwrap();
    assert!(graph.resolved_call_sites().iter().any(|site| {
        site.file == root.join("src/ownership.mts") && site.source_callee == "stage.run"
    }));
    assert!(!graph.assertion_call_sites().iter().any(|site| {
        site.file == root.join("src/ownership.mts")
            && site.source_callee == "stage.run"
            && site.target_nodes().next().is_some()
    }));
}
