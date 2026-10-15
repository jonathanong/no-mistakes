use super::*;

fn foreign_parameter_graph(order: &str) -> (PathBuf, DepGraph) {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture(&format!(
        "value-flow-foreign-parameter-{order}"
    )));
    let tsconfig = TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    let graph = DepGraph::build_with_plan(
        &root,
        &tsconfig,
        GraphBuildPlan {
            calls: true,
            ..Default::default()
        },
    )
    .unwrap();
    (root, graph)
}

#[test]
fn foreign_default_agreement_uses_all_incoming_acquisition_phases() {
    for order in ["early", "late"] {
        let (root, graph) = foreign_parameter_graph(order);
        let sites = graph.call_sites_in_file(&root.join("src/entry.mts"));
        for callee in ["outside", "inside"] {
            let site = sites
                .iter()
                .find(|site| site.source_callee == callee)
                .unwrap();
            assert!(site.targets().next().is_none(), "{order}: {site:#?}");
            assert!(site.has_unknown_target(), "{order}: {site:#?}");
        }
        // A module initializer acquired this method before any invocation.
        for callee in ["copiedOutside", "copiedInside"] {
            let site = sites
                .iter()
                .find(|site| site.source_callee == callee)
                .unwrap();
            let targets: Vec<_> = site.targets().collect();
            assert_eq!(targets.len(), 1, "{order}: {site:#?}");
            assert!(
                matches!(
                    targets[0].target,
                    KnownCallTargetRef::ModuleExport {
                        export_path: "copiedAgreementReceiver.get",
                        ..
                    }
                ),
                "{order}: {site:#?}"
            );
            assert!(!site.has_unknown_target(), "{order}: {site:#?}");
        }
    }
}

#[test]
#[cfg(feature = "test-instrumentation")]
fn repeated_foreign_default_agreement_is_cached_once_for_all_reads() {
    let (root, graph) = foreign_parameter_graph("128");
    let sites = graph.call_sites_in_file(&root.join("src/entry.mts"));
    let reads: Vec<_> = sites
        .iter()
        .filter(|site| site.source_callee == "outside" || site.source_callee.starts_with("method"))
        .collect();
    assert_eq!(reads.len(), 129);
    assert!(reads
        .iter()
        .all(|site| site.has_unknown_target() && site.targets().next().is_none()));
    assert_eq!(benchmark_probe_parameter_agreements(), 1);
}
