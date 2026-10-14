use super::*;

#[test]
fn nested_receiver_cutoffs_require_guaranteed_distinct_references() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/query-reached-per-item/provenance-fixture"),
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
    let sites = graph.call_sites_in_file(&root.join("src/nested-receiver-cutoffs.mts"));
    for name in [
        "forBody",
        "whileBody",
        "doBody",
        "conditional",
        "shortCircuit",
        "logicalAssignment",
        "loop",
        "bindingDefault",
        "arrayBindingDefault",
        "caughtReplacement",
        "cyclicAcquisition",
        "restoredReference",
        "dynamicRestoration",
        "assignmentDefault",
        "sameReceiver",
        "unknownReceiver",
        "parameterDefault",
        "acquiredBeforeGuaranteed",
    ] {
        let site = sites
            .iter()
            .find(|site| site.caller.as_deref() == Some(name) && site.source_callee == "value.get")
            .unwrap();
        assert!(
            matches!(site.target, ResolvedCallTarget::Unknown),
            "{name}: {site:?}"
        );
    }
    let guaranteed = sites
        .iter()
        .find(|site| {
            site.caller.as_deref() == Some("guaranteedAfterConditional")
                && site.source_callee == "value.get"
        })
        .unwrap();
    assert!(
        matches!(&guaranteed.target, ResolvedCallTarget::ModuleExport { export_path, .. }
        if export_path == "primary.get"),
        "{guaranteed:?}"
    );
    for name in ["savedBeforeRestoration", "savedBeforeDynamicRestoration"] {
        let site = sites
            .iter()
            .find(|site| site.caller.as_deref() == Some(name) && site.source_callee == "value.get")
            .unwrap();
        assert!(
            matches!(&site.target, ResolvedCallTarget::ModuleExport { export_path, .. } if export_path == "primary.get"),
            "{name}: {site:?}"
        );
    }
    let defaults = graph.call_sites_in_file(&root.join("src/nested-receiver-owned-defaults.mts"));
    for (name, callee, expected) in [
        ("liveDefault", "client.get", None),
        ("copiedDefault", "get", Some("primary.get")),
        ("originalArgument", "client.get", None),
        ("globalDefault", "client.get", None),
        ("earlyGlobalDefault", "client.get", Some("primary.get")),
    ] {
        let site = defaults
            .iter()
            .find(|site| site.caller.as_deref() == Some(name) && site.source_callee == callee)
            .unwrap();
        assert_eq!(
            match &site.target {
                ResolvedCallTarget::ModuleExport { export_path, .. } => Some(export_path.as_str()),
                _ => None,
            },
            expected,
            "{name}: {site:?}"
        );
    }
}
