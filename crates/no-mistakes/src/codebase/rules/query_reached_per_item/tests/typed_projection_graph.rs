use crate::codebase::dependencies::graph::{DepGraph, GraphBuildPlan, KnownCallTargetRef};
use crate::codebase::ts_resolver::TsConfig;

#[test]
fn nullish_receipts_do_not_supply_targets_to_other_branches() {
    let root = crate::test_support::rule_fixture_root("query-reached-per-item")
        .with_file_name("typed-executor-projections-fixture")
        .canonicalize()
        .unwrap();
    let config = TsConfig {
        dir: root.clone(),
        paths: Vec::new(),
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
    let file = root.join("nullish-branches.mts");
    let sites = graph.call_sites_in_file(&file);
    for (line, module, typed_module) in [
        (7, Some("./unselected.mts"), Some("./database.mts")),
        (11, Some("./database.mts"), Some("./unselected.mts")),
        (15, Some("./database.mts"), None),
        (19, None, Some("./database.mts")),
        (23, None, Some("./unselected.mts")),
        (27, Some("./unselected.mts"), None),
        (32, None, None),
    ] {
        let site = sites
            .iter()
            .find(|site| site.line == line && site.source_callee == "query")
            .unwrap_or_else(|| panic!("missing line {line}: {sites:#?}"));
        let targets = site
            .targets()
            .map(|candidate| match candidate.target {
                KnownCallTargetRef::ModuleExport {
                    specifier,
                    export_path,
                    ..
                } => (specifier, export_path),
                other => panic!("unexpected target {other:?}"),
            })
            .collect::<Vec<_>>();
        assert_eq!(
            targets,
            module
                .into_iter()
                .map(|module| (module, "write"))
                .collect::<Vec<_>>(),
            "{site:#?}"
        );
        assert_eq!(
            site.typed_from,
            typed_module
                .into_iter()
                .map(|module| (module.to_owned(), "TransactionQuery".to_owned()))
                .collect::<Vec<_>>(),
            "{site:#?}"
        );
        assert!(site.has_unknown_target(), "{site:#?}");
        assert_eq!(
            site.target_nodes().len(),
            usize::from(module.is_some()),
            "{site:#?}"
        );
    }
}

#[test]
fn nullish_alias_diamonds_share_resolution_and_receipts() {
    use crate::codebase::dependencies::graph::benchmark_probe_nullish_work;
    let fixture = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/value-flow-nullish-executor-diamond/fixture")
        .canonicalize()
        .unwrap();
    for layers in [2, 4, 8, 16] {
        let root = fixture.join(format!("layers-{layers}"));
        let config = TsConfig {
            dir: root.clone(),
            paths: Vec::new(),
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
        let sites = graph.call_sites_in_file(&root.join("entry.mts"));
        let site = sites
            .iter()
            .find(|site| site.source_callee == "query")
            .unwrap();
        assert_eq!(site.typed_from, [("./types.mts".into(), "Executor".into())]);
        assert_eq!(site.target_nodes().len(), 1, "{site:#?}");
        assert!(site.has_unknown_target());
        let [evaluations, hits, nodes, key_items, ..] = benchmark_probe_nullish_work();
        eprintln!("nullish layers={layers} evaluations={evaluations} hits={hits} nodes={nodes} key_items={key_items}");
        if layers >= 4 {
            assert!(hits > 0);
        }
        assert!(evaluations <= 32 * (layers + 1), "{evaluations}");
        assert!(nodes <= 64 * (layers + 1), "{nodes}");
        assert!(
            key_items <= 256 * (layers + 1) * (layers + 1),
            "{key_items}"
        );
    }
}

#[test]
fn ordinary_queries_leave_nullish_memo_cold() {
    let root = crate::test_support::rule_fixture_root("query-reached-per-item")
        .with_file_name("typed-client-fixture")
        .canonicalize()
        .unwrap();
    let config = TsConfig {
        dir: root.clone(),
        paths: Vec::new(),
        paths_dir: root.clone(),
        base_url: None,
    };
    DepGraph::build_with_plan(
        &root,
        &config,
        GraphBuildPlan {
            calls: true,
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(
        crate::codebase::dependencies::graph::benchmark_probe_nullish_work(),
        [0; 15]
    );
}

#[test]
fn nullish_cycles_preserve_fallbacks_and_report_work() {
    let fixture = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/value-flow-nullish-executor-cycles/fixture")
        .canonicalize()
        .unwrap();
    for case in [
        "call-cycle-diamond-2",
        "call-cycle-diamond-4",
        "call-cycle-diamond-8",
        "call-cycle-diamond-16",
        "call-cycle-diamond-32",
        "forwarding-parameters",
        "default-forwarding",
        "caller-read-overwrite",
        "entry-ancestry-forward",
        "entry-ancestry-reverse",
        "cycle-includes-diamond-2",
        "cycle-includes-diamond-4",
        "cycle-includes-diamond-8",
        "cycle-includes-diamond-16",
        "cycle-includes-diamond-32",
        "layers-0-forward",
        "layers-0-reverse",
        "layers-2-forward",
        "layers-2-reverse",
        "layers-4-forward",
        "layers-4-reverse",
        "discarded-return-cycle",
        "discarded-return-control",
    ] {
        let root = fixture.join(case);
        let config = TsConfig {
            dir: root.clone(),
            paths: Vec::new(),
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
        let sites = graph.call_sites_in_file(&root.join("entry.mts"));
        let site = sites
            .iter()
            .find(|site| site.source_callee == "query")
            .unwrap();
        let names = site
            .targets()
            .map(|candidate| match candidate.target {
                KnownCallTargetRef::RepositoryFunction { scope, .. } => scope,
                other => panic!("unexpected {other:?}"),
            })
            .collect::<Vec<_>>();
        let expected = if case == "caller-read-overwrite" {
            vec!["independent"]
        } else if case == "discarded-return-control" {
            vec!["fallbackB", "independent"]
        } else {
            vec!["fallbackA", "fallbackB", "independent"]
        };
        assert_eq!(names, expected, "case={case} {site:#?}");
        if matches!(case, "forwarding-parameters" | "caller-read-overwrite") {
            assert!(site.typed_from.is_empty());
        } else {
            assert_eq!(site.typed_from, [("./types.mts".into(), "Executor".into())]);
        }
        assert!(site.has_unknown_target());
        let work = crate::codebase::dependencies::graph::benchmark_probe_nullish_work();
        eprintln!("nullish-cycle {case} work={work:?}");
        if case.starts_with("entry-ancestry-") {
            assert!(work[10] > 0, "case={case} work={work:?}");
        }
        if let Some(layers) = case
            .strip_prefix("cycle-includes-diamond-")
            .or_else(|| case.strip_prefix("call-cycle-diamond-"))
        {
            let layers = layers.parse::<usize>().unwrap();
            assert!(work[0] <= 64 * (layers + 1), "case={case} work={work:?}");
            // Retained dependency receipts remain quadratic. Capturing a suffix
            // visits unique last-read names, rather than rescanning repeated reads.
            assert!(
                work[7] <= 16 * (layers + 1).pow(2),
                "case={case} work={work:?}"
            );
            assert!(work[8] <= 8 * (layers + 1), "case={case} work={work:?}");
            // Include guard reads and capture candidates, not just evaluations.
            assert!(
                work[4..8].iter().sum::<usize>() <= 128 * (layers + 1).pow(3),
                "case={case} work={work:?}"
            );
            assert!(
                work[8] <= 64 * (layers + 1).pow(2),
                "case={case} work={work:?}"
            );
            assert!(
                work[9] <= 64 * (layers + 1).pow(2),
                "case={case} work={work:?}"
            );
        }
    }
}
