use super::*;

type MutableCase = (&'static str, &'static str, Option<&'static str>);
#[path = "mutable_receiver_binding_cases.rs"]
mod binding_cases;
#[path = "mutable_receiver_class_cases.rs"]
mod class_cases;
#[path = "mutable_receiver_phase_cases.rs"]
mod phase_cases;

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
    for (case, callee, expected) in phase_cases::CASES
        .iter()
        .chain(class_cases::CASES)
        .chain(binding_cases::CASES)
        .copied()
    {
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
