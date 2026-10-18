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
        "provider-default",
        "provider-callback",
        "provider-for-of",
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
