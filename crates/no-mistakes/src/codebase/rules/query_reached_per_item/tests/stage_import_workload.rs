use super::*;
use crate::codebase::dependencies::graph::{CallRoot, CallTraversal, GraphBuildPlan};
use std::collections::BTreeSet;

fn expected_findings() -> BTreeSet<(String, usize, String)> {
    (0..24)
        .flat_map(|stage| {
            [
                ("src/caller.mts".into(), 28 + stage * 4, "stage.run".into()),
                (
                    "src/caller.mts".into(),
                    29 + stage * 4,
                    "stage.check".into(),
                ),
            ]
        })
        .collect()
}

#[test]
fn saved_stage_import_workload_preserves_every_finding_across_worker_counts() {
    let root = root().with_file_name("stage-import-index-workload");
    let expected = expected_findings();
    for jobs in [1, 2, 4] {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(jobs)
            .build()
            .unwrap();
        let findings = pool
            .install(|| crate::codebase::rules::run_check(&root, None, None))
            .unwrap();
        let actual = findings
            .iter()
            .map(|finding| {
                (
                    finding.file.clone(),
                    finding.line,
                    finding.import.clone().unwrap_or_default(),
                )
            })
            .collect::<BTreeSet<_>>();
        assert_eq!(
            actual,
            expected,
            "jobs={jobs}; missing={:?}; unexpected={:?}; findings={findings:#?}",
            expected.difference(&actual).collect::<Vec<_>>(),
            actual.difference(&expected).collect::<Vec<_>>()
        );
        assert_eq!(findings.len(), 48, "jobs={jobs}: {findings:#?}");
        assert!(findings.iter().all(|finding| finding.rule == RULE_ID));
    }
}

#[test]
fn saved_stage_import_workload_has_exact_caller_targets_and_typed_sinks() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &root().with_file_name("stage-import-index-workload"),
    );
    let config = crate::codebase::ts_resolver::load_tsconfig(&root.join("tsconfig.json")).unwrap();
    let graph = DepGraph::build_with_plan_and_config(
        &root,
        &config,
        GraphBuildPlan {
            calls: true,
            effect_selectors: true,
            value_import_selectors: true,
            ..Default::default()
        },
        Some(&root.join(".no-mistakes.yml")),
    )
    .unwrap();
    let caller = root.join("src/caller.mts");
    let caller_sites = graph
        .resolved_call_sites()
        .iter()
        .filter(|site| site.file == caller)
        .collect::<Vec<_>>();
    assert_eq!(caller_sites.len(), 48, "{caller_sites:#?}");
    for site in caller_sites {
        assert_eq!(site.target_nodes().count(), 1, "{site:#?}");
        assert!(!site.has_unknown_target(), "{site:#?}");
    }
    let query_sites = graph
        .resolved_call_sites()
        .iter()
        .filter(|site| site.source_callee == "query")
        .collect::<Vec<_>>();
    let expected = (0..24)
        .flat_map(|stage| {
            let file = root.join(format!("src/stages-{stage:02}.mts"));
            [8, 9].map(move |line| (file.clone(), line))
        })
        .collect::<BTreeSet<_>>();
    let actual = query_sites
        .iter()
        .map(|site| (site.file.clone(), site.line))
        .collect::<BTreeSet<_>>();
    assert_eq!(expected, actual, "{query_sites:#?}");
    assert_eq!(query_sites.len(), 96, "{query_sites:#?}");
    for site in query_sites {
        assert!(
            site.typed_from.iter().any(|(module, export)| {
                module == "./database.mts" && export == "TransactionQuery"
            }),
            "missing executor provenance: {site:#?}"
        );
    }
    let roots = graph.expand_call_roots(&[CallRoot::Function {
        file: caller,
        symbol: "cleanup".into(),
    }]);
    assert_eq!(roots.len(), 1);
    assert_eq!(
        graph
            .call_traces(&roots, CallTraversal::Transitive, None)
            .len(),
        48
    );
}
