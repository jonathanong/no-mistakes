use super::*;

fn graph() -> (DepGraph, std::path::PathBuf) {
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
    (graph, root)
}

fn assert_target(graph: &DepGraph, root: &std::path::Path, name: &str, unknown: bool) {
    let sites = graph.call_sites_in_file(&root.join(format!("src/{name}.mts")));
    let site = sites
        .iter()
        .find(|site| site.source_callee == "replica.get")
        .unwrap_or_else(|| panic!("{name} lacks replica.get: {sites:?}"));
    if unknown {
        assert!(
            site.has_unknown_target() && site.targets().next().is_none(),
            "{name}: {site:?}"
        );
    } else {
        assert!(
            matches!(&site.target, ResolvedCallTarget::ModuleExport { export_path, .. } if export_path == "replica.get"),
            "{name}: {site:?}"
        );
    }
}

#[test]
fn object_assign_projects_known_member_providers_and_keeps_controls() {
    let (graph, root) = graph();
    for name in [
        "object-assign-alias-target",
        "object-assign-alias-source",
        "object-assign-multiple-sources",
        "object-assign-nested-target",
        "object-assign-uncalled",
    ] {
        assert_target(&graph, &root, name, true);
    }
    for name in [
        "object-assign-shadowed",
        "object-assign-unrelated-property",
        "object-assign-zero-args",
        "object-assign-empty-target",
        "object-assign-non-object-source",
    ] {
        assert_target(&graph, &root, name, false);
    }
    let sites = graph.call_sites_in_file(&root.join("src/object-assign-multiple-sources.mts"));
    let unrelated = sites
        .iter()
        .find(|site| site.source_callee == "unrelated.get")
        .unwrap();
    assert!(matches!(
        &unrelated.target,
        ResolvedCallTarget::ModuleExport { export_path, .. } if export_path == "unrelated.get"
    ));
}
