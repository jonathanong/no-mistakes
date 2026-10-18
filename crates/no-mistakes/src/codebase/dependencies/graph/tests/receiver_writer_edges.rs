use super::*;

fn graph() -> (PathBuf, DepGraph) {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture(
        "foreign-receiver-declaration-order",
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
    (root, graph)
}

#[test]
fn repeated_receiver_writer_calls_invalidate_the_owners_later_provider() {
    let (root, graph) = graph();
    for name in ["repeated-writer", "repeated-writer-loop"] {
        let sites = graph.call_sites_in_file(&root.join(format!("src/{name}.mts")));
        for callee in ["value.get", "changed.get"] {
            let site = sites
                .iter()
                .find(|site| site.source_callee == callee)
                .unwrap();
            assert!(
                site.has_unknown_target() && site.targets().next().is_none(),
                "{name}: {site:?}"
            );
        }
        for (callee, expected) in [
            ("value.info", "primary.info"),
            ("changed.info", "replica.info"),
        ] {
            let site = sites
                .iter()
                .find(|site| site.source_callee == callee)
                .unwrap();
            assert!(
                matches!(&site.target, ResolvedCallTarget::ModuleExport { export_path, .. } if export_path == expected),
                "{name}: {site:?}"
            );
        }
    }
}

#[test]
fn member_rebindings_supply_future_writer_receivers_in_both_declaration_orders() {
    let (root, graph) = graph();
    for name in [
        "member-provider-writer-first",
        "member-provider-rebind-first",
        "computed-provider-writer-first",
        "computed-provider-rebind-first",
        "computed-writer-member-provider",
    ] {
        let sites = graph.call_sites_in_file(&root.join(format!("src/{name}.mts")));
        for callee in ["value.get", "changed.get"] {
            let site = sites
                .iter()
                .find(|site| site.source_callee == callee)
                .unwrap();
            assert!(
                site.has_unknown_target() && site.targets().next().is_none(),
                "{name}: {site:?}"
            );
        }
        for (callee, expected) in [
            ("value.info", "primary.info"),
            ("changed.info", "replica.info"),
        ] {
            let site = sites
                .iter()
                .find(|site| site.source_callee == callee)
                .unwrap();
            assert!(
                matches!(&site.target, ResolvedCallTarget::ModuleExport { export_path, .. } if export_path == expected),
                "{name}: {site:?}"
            );
        }
    }
}

#[test]
fn captured_sibling_calls_remain_present_and_never_follow_the_rebound_root() {
    let (root, graph) = graph();
    let sites = graph.call_sites_in_file(&root.join("src/controls.mts"));
    let site = sites
        .iter()
        .find(|site| site.source_callee == "captured.client.info")
        .expect("the statically named captured sibling call must not be dropped");
    assert_eq!(site.line, 17);
    assert!(
        site.targets().all(|target| matches!(target.target,
            KnownCallTargetRef::ModuleExport { export_path, .. } if export_path == "primary.info")),
        "{site:?}"
    );
}

#[test]
fn uncalled_writers_keep_the_documented_conservative_foreign_provider_summary() {
    let (root, graph) = graph();
    let sites = graph.call_sites_in_file(&root.join("src/uncalled.mts"));
    let site = sites
        .iter()
        .find(|site| site.source_callee == "changed.get")
        .unwrap();
    assert!(
        site.has_unknown_target() && site.targets().next().is_none(),
        "{site:?}"
    );
}

#[test]
fn opaque_and_primitive_member_stores_do_not_invent_known_targets() {
    let (root, graph) = graph();
    let sites = graph.call_sites_in_file(&root.join("src/opaque-provider-controls.mts"));
    for callee in ["unknownBox.client.get", "primitiveBox.client.get"] {
        let site = sites
            .iter()
            .find(|site| site.source_callee == callee)
            .unwrap();
        assert!(
            site.has_unknown_target() && site.targets().next().is_none(),
            "{site:?}"
        );
    }
}

#[test]
fn sibling_and_deeper_member_providers_do_not_replace_the_receiver() {
    let (root, graph) = graph();
    let sites = graph.call_sites_in_file(&root.join("src/unrelated-member-providers.mts"));
    let primary = sites
        .iter()
        .find(|site| site.source_callee == "value.get")
        .unwrap();
    assert!(primary.has_unknown_target() && primary.targets().next().is_none());
    let replica = sites
        .iter()
        .find(|site| site.source_callee == "changed.get")
        .unwrap();
    assert!(
        matches!(&replica.target,
        ResolvedCallTarget::ModuleExport { export_path, .. } if export_path == "replica.get"),
        "{replica:?}"
    );
}

#[test]
fn member_rebindings_mutate_a_nested_closures_saved_container() {
    let (root, graph) = graph();
    let sites = graph.call_sites_in_file(&root.join("src/nested-member-provider.mts"));
    let changed = sites
        .iter()
        .find(|site| site.source_callee == "changed.get")
        .unwrap();
    assert!(
        changed.has_unknown_target() && changed.targets().next().is_none(),
        "{changed:?}"
    );
    let sibling = sites
        .iter()
        .find(|site| site.source_callee == "changed.info")
        .unwrap();
    assert!(
        matches!(&sibling.target,
        ResolvedCallTarget::ModuleExport { export_path, .. } if export_path == "replica.info"),
        "{sibling:?}"
    );
}
