use super::*;

#[test]
fn provider_argument_shapes_invalidate_only_the_written_member() {
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
    for name in [
        "provider-method",
        "provider-wrapper",
        "provider-wrapper-duplicate",
        "provider-direct-callable",
        "provider-shared-call-origins",
        "provider-default",
        "provider-callback",
        "provider-callback-arrow",
        "provider-for-of",
        "provider-for-of-multiple",
        "provider-assign",
        "provider-recursive-wrapper",
    ] {
        let sites = graph.call_sites_in_file(&root.join(format!("src/{name}.mts")));
        let changed = sites
            .iter()
            .find(|site| site.source_callee == "replica.get")
            .unwrap();
        assert!(
            changed.has_unknown_target() && changed.targets().next().is_none(),
            "{name}: {changed:?}"
        );
        for expected in ["primary.info", "replica.info", "unrelated.get"] {
            let site = sites
                .iter()
                .find(|site| site.source_callee == expected)
                .unwrap();
            assert!(
                matches!(&site.target,
                ResolvedCallTarget::ModuleExport { export_path, .. } if export_path == expected),
                "{name}: {site:?}"
            );
        }
    }
}

#[test]
fn multiple_for_of_values_keep_ordinary_calls_unknown() {
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
    let ordinary = graph.call_sites_in_file(&root.join("src/provider-for-of-ordinary.mts"));
    let loop_call = ordinary
        .iter()
        .find(|site| site.source_callee == "client.get")
        .unwrap();
    assert!(loop_call.has_unknown_target() && loop_call.targets().next().is_none());
    for expected in ["primary.info", "replica.info"] {
        let site = ordinary
            .iter()
            .find(|site| site.source_callee == expected)
            .unwrap();
        assert!(matches!(
            &site.target,
            ResolvedCallTarget::ModuleExport { export_path, .. } if export_path == expected
        ));
    }
    let provider = graph.call_sites_in_file(&root.join("src/provider-for-of-multiple.mts"));
    let primary = provider
        .iter()
        .find(|site| site.source_callee == "primary.get")
        .unwrap();
    assert!(primary.has_unknown_target() && primary.targets().next().is_none());
}

#[test]
fn unsupported_for_of_heads_do_not_mutate_imported_methods() {
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
    let sites = graph.call_sites_in_file(&root.join("src/provider-for-of-unsupported.mts"));
    for expected in ["primary.get", "replica.get"] {
        let site = sites
            .iter()
            .find(|site| site.source_callee == expected)
            .unwrap();
        assert!(matches!(
            &site.target,
            ResolvedCallTarget::ModuleExport { export_path, .. } if export_path == expected
        ));
    }
}
