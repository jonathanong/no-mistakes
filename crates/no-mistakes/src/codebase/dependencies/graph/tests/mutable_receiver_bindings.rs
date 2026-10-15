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
        ("mutable-let-pattern-class-context", "value.get", None),
        (
            "mutable-let-pattern-class-context",
            "changed.get",
            Some("replica.get"),
        ),
        ("mutable-let-pattern-final-foreign", "changed.get", None),
        (
            "mutable-let-pattern-conditional-whole",
            "value.get",
            Some("primary.get"),
        ),
        ("mutable-let-pattern-conditional-final", "value.get", None),
        ("mutable-let-pattern-conditional-final", "changed.get", None),
        ("mutable-let-pattern-cold-class", "value.get", None),
        (
            "mutable-let-pattern-cold-class",
            "changed.get",
            Some("replica.get"),
        ),
        ("mutable-let-pattern-shadowed-class", "value.get", None),
        (
            "mutable-let-pattern-shadowed-class",
            "changed.get",
            Some("replica.get"),
        ),
        (
            "mutable-let-pattern-final-root",
            "value.get",
            Some("primary.get"),
        ),
        ("mutable-let-pattern-final-root", "changed.get", None),
        (
            "mutable-let-pattern-foreign-catalog",
            "value.get",
            Some("primary.get"),
        ),
        ("mutable-let-pattern-logical-rebind", "value.get", None),
        ("mutable-let-pattern-logical-rebind", "changed.get", None),
        ("mutable-let-pattern-foreign-rebind", "changed.get", None),
        (
            "mutable-let-pattern-shorthand",
            "value.get",
            Some("primary.get"),
        ),
        ("mutable-let-pattern-rhs-rebind", "changed.get", None),
        (
            "mutable-let-pattern-lhs-rebind",
            "value.get",
            Some("primary.get"),
        ),
        ("mutable-let-pattern-lhs-rebind", "changed.get", None),
        ("mutable-let-pattern-conditional-rebind", "value.get", None),
        (
            "mutable-let-pattern-conditional-rebind",
            "changed.get",
            None,
        ),
        ("mutable-let-pattern-self-copy", "value.get", None),
        ("mutable-let-pattern-nested-before-rhs", "value.get", None),
        (
            "mutable-let-pattern-rhs-rebind",
            "value.get",
            Some("primary.get"),
        ),
        (
            "mutable-let-pattern-twice",
            "value.get",
            Some("primary.get"),
        ),
        ("mutable-let-pattern-before", "value.get", None),
        (
            "mutable-let-wrapped-unrelated",
            "value.get",
            Some("primary.get"),
        ),
        (
            "mutable-let-pattern-wrapped",
            "value.get",
            Some("primary.get"),
        ),
        ("mutable-let-pattern-lhs", "value.get", Some("primary.get")),
        ("mutable-let-pattern-rhs", "value.get", None),
        (
            "mutable-let-pattern-default",
            "value.get",
            Some("primary.get"),
        ),
        ("mutable-let-wrapped-as", "value.get", Some("primary.get")),
        (
            "mutable-let-wrapped-non-null",
            "value.get",
            Some("primary.get"),
        ),
        (
            "mutable-let-wrapped-assertion",
            "value.get",
            Some("primary.get"),
        ),
        (
            "mutable-let-wrapped-satisfies",
            "value.get",
            Some("primary.get"),
        ),
        (
            "mutable-let-destructured-array",
            "value.get",
            Some("primary.get"),
        ),
        (
            "mutable-let-destructured-object",
            "value.get",
            Some("primary.get"),
        ),
        (
            "mutable-let-destructured-default",
            "value.get",
            Some("primary.get"),
        ),
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
