use super::*;

#[test]
fn one_time_lexical_member_replacements_invalidate_direct_imported_calls() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture(
        "value-flow-lexical-member-replacement",
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
    let sites = graph.call_sites_in_file(&root.join("src/entry.mts"));
    let primary: Vec<_> = sites
        .iter()
        .filter(|site| site.source_callee == "primary.get")
        .collect();
    assert_eq!(primary.len(), 2);
    assert!(!primary[0].has_unknown_target());
    assert!(primary[1].has_unknown_target(), "{:?}", primary[1]);
    assert!(primary[1].targets().next().is_none());
    assert!(primary[1].target_nodes().next().is_none());
    for callee in ["dynamic.get", "opaque.get"] {
        let site = sites
            .iter()
            .find(|site| site.source_callee == callee)
            .unwrap();
        assert!(site.target_nodes().next().is_none());
        assert!(
            site.has_unknown_target() && site.targets().next().is_none(),
            "{site:?}"
        );
    }
    for (callee, expected) in [
        ("saved", "primary.get"),
        ("primary.info", "primary.info"),
        ("untouched.get", "untouched.get"),
    ] {
        let site = sites
            .iter()
            .find(|site| site.source_callee == callee)
            .unwrap();
        let targets: Vec<_> = site.targets().collect();
        assert_eq!(targets.len(), 1, "{site:?}");
        assert!(
            matches!(targets[0].target, KnownCallTargetRef::ModuleExport { export_path, .. } if export_path == expected),
            "{site:?}"
        );
        assert!(!site.has_unknown_target());
    }
    let minimal = graph.call_sites_in_file(&root.join("src/minimal.mts"));
    let site = minimal
        .iter()
        .find(|site| site.source_callee == "primary.get")
        .unwrap();
    assert!(
        site.has_unknown_target() && site.targets().next().is_none(),
        "{site:?}"
    );
    let after = graph.call_sites_in_file(&root.join("src/after.mts"));
    let site = after
        .iter()
        .find(|site| site.source_callee == "primary.get")
        .unwrap();
    assert!(!site.has_unknown_target(), "{site:?}");
}
