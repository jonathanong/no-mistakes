use super::*;

#[test]
fn dynamic_bases_follow_policy_without_guessing_error_ancestry() {
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
        let result: serde_json::Value = serde_json::from_str(&json).unwrap();
        let rules = result["rules"].as_array().unwrap();
        let advisories = result["advisories"].as_array().unwrap();
        assert_eq!(result["warnings"], serde_json::json!([]));
        let definite: Vec<_> = rules
            .iter()
            .filter(|finding| finding["target"] != "unanalyzable")
            .map(|finding| finding["target"].as_str().unwrap())
            .collect();
        assert_eq!(definite, ["DefinitelyDead"], "{policy}: {result}");
        let uncertain: Vec<_> = rules
            .iter()
            .chain(advisories)
            .filter(|finding| finding["target"] == "unanalyzable")
            .collect();
        let names = [
            "MixedError",
            "MixedBase",
            "ExpressionBase",
            "ComputedBase",
            "StaticChild",
            "Errors.Dynamic",
            "TypeOnly",
        ];
        assert_eq!(
            uncertain.len(),
            if policy == "ignore" { 0 } else { names.len() },
            "{policy}: {result}"
        );
        if policy != "ignore" {
            for name in names {
                assert!(
                    uncertain.iter().any(|finding| finding["message"]
                        .as_str()
                        .unwrap()
                        .contains(&format!("`{name}`"))),
                    "{name}: {result}"
                );
            }
            let type_only = uncertain
                .iter()
                .find(|finding| finding["file"] == "src/type-only.ts")
                .unwrap();
            assert_eq!(type_only["line"], 1);
        }
        assert_eq!(
            rules.len(),
            if policy == "report" { 8 } else { 1 },
            "{result}"
        );
        assert_eq!(
            advisories.len(),
            if policy == "advise" { 7 } else { 0 },
            "{result}"
        );
        assert_eq!(
            result["suppressed"].as_array().unwrap().len(),
            if policy == "ignore" { 0 } else { 1 },
            "{result}"
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
    let mixed = with
        .class_declarations()
        .iter()
        .find(|class| class.scope == "MixedError")
        .unwrap();
    assert_eq!(mixed.line, 4);
    assert!(mixed.exported);
    assert_eq!(mixed.global_base, None);
    assert!(with
        .dependencies_of_node(&mixed.node())
        .into_iter()
        .flatten()
        .all(|(_, kind)| *kind != EdgeKind::Extends));
    assert!(build(true, false).resolved_call_sites().is_empty());
}
