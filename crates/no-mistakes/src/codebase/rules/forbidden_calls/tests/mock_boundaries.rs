use super::*;

fn run_mock_case(name: &str) -> Vec<crate::codebase::rules::RuleFinding> {
    let base = crate::test_support::rule_fixture_root("forbidden-calls");
    let base = base.parent().unwrap();
    run_check(
        &base.join(if name == "suppressed" {
            "mock-boundaries-suppression/fixture"
        } else if name == "manual" {
            "mock-manual/fixture"
        } else {
            "mock-boundaries/fixture"
        }),
        Some(&base.join(format!("configs/mock-boundaries-{name}.yml"))),
        None,
    )
    .unwrap()
}

#[test]
fn full_mock_boundaries_replace_the_real_module_per_root() {
    for name in [
        "full",
        "auto",
        "jest",
        "jest-require",
        "actual-other",
        "module-full",
        "function-full",
        "suppressed",
    ] {
        let findings = run_mock_case(name);
        assert!(findings.is_empty(), "{name}: {findings:#?}");
    }
    for name in [
        "real",
        "partial",
        "original",
        "nonliteral",
        "domock",
        "different",
        "duplicate",
        "captured",
        "shadow",
        "root-isolation",
        "actual-outside",
        "dynamic-bypass",
        "helper",
        "helper-external-value",
        "require-actual",
        "jest-require-real",
        "module-real",
        "function-real",
        "external-factory",
        "external-value",
        "global-factory",
    ] {
        let findings = run_mock_case(name);
        assert_eq!(findings.len(), 1, "{name}: {findings:#?}");
        assert!(
            findings[0].file.ends_with("src/analysis.mts"),
            "{name}: {findings:#?}"
        );
    }
}

#[test]
fn full_mock_factory_is_walked_instead_of_the_real_module() {
    let findings = run_mock_case("factory");
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert!(
        findings[0].file.ends_with("src/factory.mts"),
        "{findings:#?}"
    );
}

#[test]
fn root_mock_boundaries_have_node_api_parity() {
    let fixture = crate::test_support::rule_fixture_root("forbidden-calls");
    let base = fixture.parent().unwrap();
    for (name, expected) in [
        ("full", 0),
        ("dynamic-bypass", 1),
        ("real", 1),
        ("factory", 1),
        ("root-isolation", 1),
        ("initializer", 1),
        ("initializer-dependency", 1),
        ("manual", 3),
    ] {
        let fixture = if name == "manual" {
            "mock-manual/fixture"
        } else {
            "mock-boundaries/fixture"
        };
        let input = crate::napi_api::options::test_json_arg(serde_json::json!({
            "root": base.join(fixture),
            "config": base.join(format!("configs/mock-boundaries-{name}.yml")),
        }));
        let report: serde_json::Value =
            serde_json::from_str(&crate::napi_api::check_json_impl(input).unwrap()).unwrap();
        #[cfg(feature = "test-instrumentation")]
        {
            let aggregate: serde_json::Value = serde_json::from_str(
                &crate::napi_api::analyze_project_json_impl(
                    crate::napi_api::options::test_json_arg(serde_json::json!({
                        "root": base.join(fixture),
                        "config": base.join(format!("configs/mock-boundaries-{name}.yml")),
                        "reports": [{"type": "check"}],
                    })),
                )
                .unwrap(),
            )
            .unwrap();
            assert_eq!(aggregate["reports"][0]["result"], report, "{name}");
        }
        assert_eq!(
            report["rules"].as_array().unwrap().len(),
            expected,
            "{name}: {report}"
        );
    }
}

#[test]
fn self_mock_does_not_remove_the_declaring_root() {
    let findings = run_mock_case("self");
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert!(findings[0].file.ends_with("src/self.mts"), "{findings:#?}");
}

#[test]
fn factory_bodies_are_selected_for_module_and_function_roots() {
    for name in ["module-factory", "function-factory"] {
        let findings = run_mock_case(name);
        assert_eq!(findings.len(), 1, "{name}: {findings:#?}");
        assert!(
            findings[0].file.contains("factory.mts"),
            "{name}: {findings:#?}"
        );
    }
}

#[test]
fn mock_selection_does_not_prune_the_canonical_call_graph() {
    use crate::codebase::dependencies::graph::{CallRoot, CallTraversal, DepGraph, GraphBuildPlan};
    let base = crate::test_support::rule_fixture_root("forbidden-calls");
    let root = base.parent().unwrap().join("mock-boundaries/fixture");
    let config = crate::codebase::ts_resolver::TsConfig {
        dir: root.clone(),
        paths_dir: root.clone(),
        ..Default::default()
    };
    let graph = DepGraph::build_with_plan(
        &root,
        &config,
        GraphBuildPlan {
            calls: true,
            imports: true,
            mock_boundaries: true,
            ..Default::default()
        },
    )
    .unwrap();
    let roots = graph.expand_call_roots(&[CallRoot::File(root.join("src/full.mts"))]);
    let ordinary = graph.call_source_nodes(&roots, CallTraversal::Transitive, None);
    assert!(ordinary.iter().any(|node| node
        .as_file()
        .is_some_and(|file| file.ends_with("src/analysis.mts"))));
    let mocked = graph.call_source_nodes_with_root_mocks(&roots, None);
    assert!(!mocked.iter().any(|node| node
        .as_file()
        .is_some_and(|file| file.ends_with("src/analysis.mts"))));
    assert_eq!(
        ordinary,
        graph.call_source_nodes(&roots, CallTraversal::Transitive, None)
    );
}

#[test]
fn mixed_mock_roots_share_one_ordinary_source_union() {
    use crate::codebase::dependencies::graph::{CallRoot, CallTraversal, DepGraph, GraphBuildPlan};
    let base = crate::test_support::rule_fixture_root("forbidden-calls");
    let root = base.parent().unwrap().join("mock-boundaries/fixture");
    let config = crate::codebase::ts_resolver::TsConfig {
        dir: root.clone(),
        paths_dir: root.clone(),
        ..Default::default()
    };
    let graph = DepGraph::build_with_plan(
        &root,
        &config,
        GraphBuildPlan {
            calls: true,
            imports: true,
            mock_boundaries: true,
            ..Default::default()
        },
    )
    .unwrap();
    let ordinary = graph.expand_call_roots(&[
        CallRoot::File(root.join("src/real.mts")),
        CallRoot::File(root.join("src/shadow.mts")),
    ]);
    let expected = graph.call_source_nodes(&ordinary, CallTraversal::Transitive, None);
    let mut mixed = ordinary;
    mixed.extend(graph.expand_call_roots(&[CallRoot::File(root.join("src/full.mts"))]));
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let selected = crate::diagnostics::with_observer(Some(observer.clone()), || {
        graph.call_source_nodes_with_root_mocks(&mixed, None)
    });
    assert!(expected.is_subset(&selected));
    assert_eq!(
        observer
            .snapshot()
            .work
            .get("graph.call_source_nodes.expanded"),
        Some(&(expected.len() as u64)),
        "ordinary roots must share one traversal even when a mocked root is present"
    );
}

#[test]
fn nonhoisted_factories_require_reachable_registration_owners() {
    for name in [
        "registration-unused-module",
        "registration-unused-function",
        "registration-depth-limit",
    ] {
        let findings = run_mock_case(name);
        assert!(findings.is_empty(), "{name}: {findings:#?}");
    }
    for name in [
        "registration-reachable-module",
        "registration-reachable-function",
        "registration-depth-reached",
    ] {
        let findings = run_mock_case(name);
        assert_eq!(findings.len(), 1, "{name}: {findings:#?}");
        assert!(
            findings[0].file.ends_with("src/registration-reachable.mts"),
            "{name}: {findings:#?}"
        );
    }
}

#[test]
fn nested_mock_factories_remain_hoisted_for_narrow_roots() {
    for name in [
        "registration-hoisted-module",
        "registration-hoisted-function",
    ] {
        let findings = run_mock_case(name);
        assert_eq!(findings.len(), 1, "{name}: {findings:#?}");
        assert!(
            findings[0].file.ends_with("src/registration-hoisted.mts"),
            "{name}: {findings:#?}"
        );
    }
}

#[test]
fn manual_automocks_retain_real_and_replacement_findings() {
    assert!(run_mock_case("auto").is_empty());
    let findings = run_mock_case("manual");
    assert_eq!(findings.len(), 3, "{findings:#?}");
    assert!(
        findings
            .iter()
            .any(|finding| finding.file.ends_with("src/analysis.mts")),
        "{findings:#?}"
    );
    assert!(
        findings
            .iter()
            .any(|finding| finding.file.ends_with("src/__mocks__/analysis.mts")),
        "{findings:#?}"
    );
    assert!(findings
        .iter()
        .any(|finding| finding.file.ends_with("src/manual-init.mts")));
}
