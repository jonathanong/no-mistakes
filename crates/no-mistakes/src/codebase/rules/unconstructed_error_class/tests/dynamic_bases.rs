use super::*;

// Output cases moved to cli_rra_snapshot_unconstructed_error_class.

#[test]
fn dynamic_bases_audit_preserves_suppressed_channel() {
    let root = fixture().parent().unwrap().join("dynamic-bases/fixture");
    for policy in ["report", "advise", "ignore"] {
        let json = crate::napi_api::check_json_impl(crate::napi_api::options::test_json_arg(
            serde_json::json!({
                "root": root, "config": root.join(format!("{policy}.yml")),
                "includeSuppressed": true,
            })
            .to_string(),
        ))
        .unwrap();
        let result: serde_json::Value = crate::test_support::without_hygiene_advisory_json(
            serde_json::from_str(&json).unwrap(),
        );
        assert_eq!(
            result["suppressed"].as_array().unwrap().len(),
            if policy == "ignore" { 0 } else { 1 },
            "{policy}: {result}"
        );
    }
}

#[test]
fn dynamic_ancestry_is_additive_without_synthetic_source_calls() {
    use crate::codebase::dependencies::graph::{DepGraph, EdgeKind, GraphBuildPlan};
    use crate::codebase::ts_resolver::TsConfig;
    let root = fixture().parent().unwrap().join("dynamic-bases/fixture");
    let config = TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    let build = |extends, calls| {
        DepGraph::build_with_plan(
            &root,
            &config,
            GraphBuildPlan {
                extends,
                calls,
                ..Default::default()
            },
        )
        .unwrap()
    };
    let with = build(true, true);
    let without = build(false, true);
    assert_eq!(with.resolved_call_sites(), without.resolved_call_sites());
    assert!(without.class_declarations().is_empty());
    assert!(with
        .class_declarations()
        .iter()
        .all(|class| class.scope != "Expression"));
    let mixed = with
        .class_declarations()
        .iter()
        .find(|class| class.scope == "MixedError")
        .unwrap();
    assert_eq!(mixed.line, 4);
    assert!(mixed.exported);
    assert_eq!(mixed.global_base, None);
    assert!(mixed.dynamic_base);
    let static_child = with
        .class_declarations()
        .iter()
        .find(|class| class.scope == "StaticChild")
        .unwrap();
    assert!(!static_child.dynamic_base);
    assert!(with
        .dependencies_of_node(&mixed.node())
        .into_iter()
        .flatten()
        .all(|(_, kind)| *kind != EdgeKind::Extends));
    assert!(build(true, false).resolved_call_sites().is_empty());
}
