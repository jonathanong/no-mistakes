use super::*;

#[test]
fn foreign_receiver_origins_do_not_follow_callable_declaration_order() {
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
        "writer-first",
        "closure-writer-first",
        "closure-rebind-first",
        "closure-live-capture",
        "closure-writers-16",
        "closure-writers-64",
        "rebind-first",
        "reverse-calls",
        "uncalled",
        "deferred",
        "escaped",
        "cycle",
        "root-cycle",
        "mixed-writes",
        "multiple-origins",
        "scale-16",
        "scale-256",
    ] {
        let sites = graph.call_sites_in_file(&root.join(format!("src/{name}.mts")));
        for callee in ["value.get", "changed.get"]
            .into_iter()
            .filter(|callee| !name.starts_with("scale-") || *callee == "changed.get")
            .filter(|callee| !matches!(name, "uncalled" | "deferred") || *callee != "value.get")
        {
            let site = sites
                .iter()
                .find(|site| site.source_callee == callee)
                .unwrap();
            assert!(
                matches!(site.target, ResolvedCallTarget::Unknown),
                "{name}: {site:?}"
            );
            assert!(site.alternatives.is_empty(), "{name}: {site:?}");
            assert!(site.has_unknown_target() && site.targets().next().is_none());
        }
        let incoming = sites
            .iter()
            .find(|site| site.source_callee == "consume")
            .unwrap();
        let target = incoming
            .target_node
            .as_ref()
            .expect("canonical consume node");
        assert!(graph
            .dependents_of_node(target)
            .into_iter()
            .flatten()
            .any(|(_, kind)| *kind == EdgeKind::Call));
    }
    // Returned nested writers have no closed module-binding phase proof. Once
    // their mutations are visible, affected copied methods stay conservative.
    let sites = graph.call_sites_in_file(&root.join("src/closure-controls.mts"));
    for callee in ["before", "after"] {
        let site = sites
            .iter()
            .find(|site| site.source_callee == callee)
            .unwrap();
        assert!(site.has_unknown_target() && site.targets().next().is_none());
    }
    // A closed canonical caller chain with no invocation leaves the module's
    // initial provider unchanged. Foreign callable providers stay conservative.
    for (file, callee, export) in [
        ("uncalled", "value.get", "primary.get"),
        ("deferred", "value.get", "primary.get"),
        ("controls", "value.info", "primary.info"),
        ("controls", "changed.info", "replica.info"),
        ("controls", "other.get", "unrelated.get"),
        ("controls", "method", "replica.get"),
        ("controls", "original", "primary.get"),
        ("cold", "value.get", "primary.get"),
        ("closure-module-captured", "changed.get", "replica.get"),
        ("closure-same-owner", "changed.get", "replica.get"),
        ("closure-controls", "value.info", "primary.info"),
        ("closure-controls", "changed.info", "replica.info"),
        ("closure-controls", "other.get", "unrelated.get"),
        ("captured-root", "stable.get", "replica.get"),
    ] {
        let sites = graph.call_sites_in_file(&root.join(format!("src/{file}.mts")));
        let site = sites
            .iter()
            .find(|site| site.source_callee == callee)
            .unwrap();
        assert!(
            matches!(&site.target, ResolvedCallTarget::ModuleExport { export_path, .. } if export_path == export),
            "{file}: {site:?}"
        );
    }
}
