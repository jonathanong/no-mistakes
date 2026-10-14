use super::*;

#[test]
fn module_scope_lazy_receiver_reads_follow_write_order_and_preserve_snapshots() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture(
        "value-flow-module-scope-lazy-alias",
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
    for (file, callee, occurrence, expected) in [
        ("lazy-read", "options.client.get", 0, Some("primary.get")),
        ("lazy-read", "options.client.get", 1, None),
        ("direct-alias", "alias.get", 0, None),
        ("snapshot-and-unrelated", "copied", 0, Some("primary.get")),
        (
            "snapshot-and-unrelated",
            "options.client.info",
            0,
            Some("primary.info"),
        ),
    ] {
        let sites = graph.call_sites_in_file(&root.join(format!("src/{file}.mts")));
        let site = sites
            .iter()
            .filter(|site| site.source_callee == callee)
            .nth(occurrence)
            .unwrap();
        match expected {
            Some(expected) => assert!(
                matches!(&site.target, ResolvedCallTarget::ModuleExport { export_path, .. }
                    if export_path == expected),
                "{file} {callee}: {site:?}"
            ),
            None => assert!(
                matches!(site.target, ResolvedCallTarget::Unknown),
                "{file} {callee}: {site:?}"
            ),
        }
    }
}

#[cfg(feature = "test-instrumentation")]
#[test]
fn module_scope_lazy_reads_keep_origin_work_linear_without_parameter_cache_entries() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture(
        "value-flow-module-scope-lazy-alias",
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
    let sites = graph.call_sites_in_file(&root.join("src/many-lazy-reads.mts"));
    assert_eq!(sites.len(), 128);
    assert!(sites.iter().all(|site| {
        site.source_callee == "options.client.get"
            && matches!(site.target, ResolvedCallTarget::Unknown)
    }));
    assert_eq!(benchmark_probe_parameter_agreements(), 0);
    assert!(
        benchmark_probe_origin_nodes() <= 128 * 8 + 8,
        "module-scope reads must keep origin work linear: {} nodes",
        benchmark_probe_origin_nodes()
    );
}
