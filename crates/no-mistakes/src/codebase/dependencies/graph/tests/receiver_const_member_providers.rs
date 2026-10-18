fn assert_member_store_receiver_cases(names: &[&str]) {
    let (root, graph) = graph();
    let mut stale_targets = Vec::new();
    for name in names {
        let sites = graph.call_sites_in_file(&root.join(format!("src/{name}.mts")));
        let changed = sites
            .iter()
            .find(|site| site.source_callee == "replica.get")
            .unwrap();
        if !changed.has_unknown_target() || changed.targets().next().is_some() {
            stale_targets.push(format!("{name}: {changed:?}"));
        }
        // Sibling and unrelated targets must remain specifically Known.
        for (callee, expected) in [
            ("primary.info", "primary.info"),
            ("replica.info", "replica.info"),
            ("unrelated.get", "unrelated.get"),
        ] {
            let site = sites
                .iter()
                .find(|site| site.source_callee == callee)
                .unwrap();
            assert!(
                matches!(&site.target,
                ResolvedCallTarget::ModuleExport { export_path, .. } if export_path == expected),
                "{name}: {site:?}"
            );
        }
    }
    assert!(stale_targets.is_empty(), "{stale_targets:#?}");
}

#[test]
fn const_member_stores_are_foreign_receiver_providers_in_both_orders() {
    assert_member_store_receiver_cases(&[
        "const-member-writer-first",
        "const-member-rebind-first",
        "const-member-repeated-writer",
    ]);
}

#[test]
fn parameter_rhs_member_stores_supply_const_and_let_receivers() {
    assert_member_store_receiver_cases(&[
        "parameter-member-const-writer-first",
        "parameter-member-const-rebind-first",
        "parameter-member-let-writer-first",
        "parameter-member-let-rebind-first",
    ]);
}

#[test]
fn direct_arrow_member_provider_keeps_unused_arguments_and_readonly_patterns_known() {
    assert_member_store_receiver_cases(&["parameter-member-arrow-provider"]);
    let (root, graph) = graph();
    let sites = graph.call_sites_in_file(&root.join("src/parameter-member-arrow-provider.mts"));
    let sibling = sites.iter().find(|site| site.source_callee == "sibling.get").unwrap();
    assert!(matches!(&sibling.target,
        ResolvedCallTarget::ModuleExport { export_path, .. } if export_path == "unrelated.get"),
        "{sibling:?}");
}

#[test]
fn module_member_stores_supply_const_and_let_writer_receivers() {
    assert_member_store_receiver_cases(&["module-member-const", "module-member-let"]);
}

#[test]
fn inactive_module_member_providers_preserve_known_calls() {
    let (root, graph) = graph();
    for name in [
        "module-member-before-provider",
        "module-member-overwritten-provider",
        "module-member-partial-overwritten-provider",
    ] {
        let sites = graph.call_sites_in_file(&root.join(format!("src/{name}.mts")));
        for expected in ["replica.get", "primary.info"] {
            let site = sites.iter().find(|site| site.source_callee == expected).unwrap();
            assert!(matches!(&site.target,
                ResolvedCallTarget::ModuleExport { export_path, .. } if export_path == expected),
                "{name}: {site:?}");
        }
    }
}
