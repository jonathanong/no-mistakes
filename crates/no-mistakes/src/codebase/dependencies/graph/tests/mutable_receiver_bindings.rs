use super::*;

#[test]
fn mutable_receiver_origins_invalidate_without_becoming_value_bindings() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &crate::test_support::rule_fixture_root("query-reached-per-item")
            .with_file_name("provenance-fixture"),
    );
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
    for (case, callee, expected) in [
        (
            "mutable-let-rebound-detaches-old",
            "value.get",
            Some("primary.get"),
        ),
        ("mutable-let-self-copy", "value.get", None),
        ("mutable-let-conditional-root", "value.get", None),
        ("mutable-let-logical-root", "value.get", None),
        ("mutable-let-lhs-acquisition", "value.get", None),
        ("mutable-let-foreign-root", "value.get", None),
        ("mutable-let-guarded", "value.get", None),
        ("mutable-const-guarded", "value.get", None),
        ("mutable-let-direct", "value.get", None),
        ("mutable-let-array", "value.get", None),
        ("mutable-let-detached", "value.get", Some("primary.get")),
        ("mutable-let-sibling", "value.get", Some("primary.get")),
        ("mutable-let-rebound", "value.get", None),
        ("mutable-let-rhs-rebound", "value.get", None),
        ("mutable-let-rhs-named", "value.get", None),
        ("mutable-let-shadowed", "value.get", Some("primary.get")),
        ("mutable-let-cycle", "value.get", None),
        ("mutable-let-copied", "fn", Some("primary.get")),
    ] {
        let sites = graph
            .call_sites_in_file(&root.join(format!("src/nested-receiver-containers/{case}.mts")));
        let site = sites
            .iter()
            .find(|site| site.caller.as_deref() == Some("consume") && site.source_callee == callee)
            .unwrap();
        match expected {
            Some(expected) => assert!(
                matches!(&site.target, ResolvedCallTarget::ModuleExport { export_path, .. } if export_path == expected),
                "{case}: {site:?}"
            ),
            None => assert!(
                matches!(site.target, ResolvedCallTarget::Unknown),
                "{case}: {site:?}"
            ),
        }
    }
}
