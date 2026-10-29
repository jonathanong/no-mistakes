use super::*;

fn run_external_chains(config: &str) -> Vec<crate::codebase::rules::RuleFinding> {
    let fixture = crate::test_support::rule_fixture_root("forbidden-calls");
    let base = fixture.parent().unwrap();
    run_check(
        &base.join("external-chains/fixture"),
        Some(&base.join(format!("configs/external-chains-{config}.yml"))),
        None,
    )
    .unwrap()
}

#[test]
fn external_chains_keep_unknown_diagnostics_without_a_configured_target() {
    let findings = run_external_chains("unknown");
    let mut imports: Vec<_> = findings
        .iter()
        .map(|finding| finding.import.as_deref())
        .collect();
    imports.sort();
    assert_eq!(
        imports,
        [
            Some("local.a.b"),
            Some("ns.a.b"),
            Some("ns.default.b"),
            Some("value.a.b")
        ],
        "{findings:#?}"
    );
    assert!(findings
        .iter()
        .all(|finding| finding.target.as_deref() == Some("unknown call")));
}

#[test]
fn external_chains_still_match_configured_module_exports_when_unknowns_are_ignored() {
    for policy in ["configured", "ignore"] {
        let findings = run_external_chains(policy);
        for (callee, target) in [
            ("ns.a.b", "module export `some-pkg#a.b`"),
            ("value.a.b", "module export `some-pkg#default.a.b`"),
            ("ns.default.b", "module export `some-pkg#default.b`"),
            ("value.b", "module export `some-pkg#default.b`"),
        ] {
            assert!(
                findings
                    .iter()
                    .any(|finding| finding.import.as_deref() == Some(callee)
                        && finding.target.as_deref() == Some(target)),
                "{policy}: {findings:#?}"
            );
        }
        assert_eq!(
            findings.len(),
            if policy == "ignore" { 4 } else { 5 },
            "{policy}: {findings:#?}"
        );
    }
}

#[test]
fn external_chains_keep_fallback_provenance_through_direct_aliases() {
    let findings = run_external_chains("aliases");
    let mut imports: Vec<_> = findings
        .iter()
        .map(|finding| finding.import.as_deref())
        .collect();
    imports.sort();
    assert_eq!(
        imports,
        [Some("defaultFull"), Some("nsFull")],
        "{findings:#?}"
    );
}

#[test]
fn external_chain_provenance_does_not_leak_to_another_call_at_the_same_offset() {
    use crate::codebase::dependencies::graph::{DepGraph, GraphBuildPlan, ResolvedCallTarget};
    let fixture = crate::test_support::rule_fixture_root("forbidden-calls");
    let root = fixture.parent().unwrap().join("external-chains/fixture");
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
            ..Default::default()
        },
    )
    .unwrap();
    let site = graph
        .resolved_call_sites()
        .iter()
        .find(|site| site.source_callee == "ns.a.b")
        .unwrap();
    assert!(graph.opaque_external_member_call(site));
    let mut other = site.clone();
    other.source_callee = "differentCallee".into();
    assert!(!graph.opaque_external_member_call(&other));
    other = site.clone();
    other.offset += 1;
    assert!(!graph.opaque_external_member_call(&other));
    other = site.clone();
    other.target = ResolvedCallTarget::Global {
        name: "globalCall".into(),
    };
    assert!(!graph.opaque_external_member_call(&other));
}

#[test]
fn external_chain_unknown_policy_has_node_api_parity() {
    let fixture = crate::test_support::rule_fixture_root("forbidden-calls");
    let base = fixture.parent().unwrap();
    for (policy, expected) in [("unknown", 4), ("configured", 5), ("ignore", 4)] {
        let input = crate::napi_api::options::test_json_arg(serde_json::json!({
            "root": base.join("external-chains/fixture"),
            "config": base.join(format!("configs/external-chains-{policy}.yml")),
        }));
        let report: serde_json::Value =
            serde_json::from_str(&crate::napi_api::check_json_impl(input).unwrap()).unwrap();
        assert_eq!(
            report["rules"].as_array().unwrap().len(),
            expected,
            "{report}"
        );
    }
}
