use super::*;
fn fixture() -> std::path::PathBuf {
    crate::test_support::rule_fixture_root("coverage-producer-assertion-owner")
}
fn run(name: &str) -> Vec<RuleFinding> {
    let root = fixture();
    crate::codebase::rules::run_check(&root, Some(&root.join(name)), None)
        .unwrap()
        .into_iter()
        .filter(|finding| finding.rule == RULE_ID)
        .collect()
}
#[test]
fn configured_producers_require_actual_reachable_assertion_owners() {
    let findings = run(".no-mistakes.yml");
    let symbols = findings
        .iter()
        .filter_map(|finding| finding.target.as_deref())
        .collect::<Vec<_>>();
    for accepted in [
        "direct",
        "imported",
        "callback",
        "crossProcess",
        "hoistedAfterReturn",
        "inlineCallback",
    ] {
        assert!(!symbols.contains(&accepted), "{findings:#?}");
    }
    for rejected in [
        "productionOnly",
        "unreachable",
        "dynamic",
        "unmodeledProcess",
        "emptyContract",
        "deadBranch",
        "afterReturn",
        "deadWhile",
        "deadFor",
        "deadConditional",
        "deadAnd",
        "deadOr",
        "deadNullish",
        "lexicalSibling",
        "empty",
        "cycle",
        "afterBreak",
        "afterContinue",
        "registeredCallback",
        "memberOnly",
        "constructOnly",
        "deadProcess",
        "nestedDeadRanges",
    ] {
        assert!(symbols.contains(&rejected), "{findings:#?}");
    }
    for finding in &findings {
        assert_eq!(finding.file, "src/producers.mts");
        assert!(finding.line > 0);
        assert!(finding.message.contains("assertion") || finding.message.contains("unanalyzable"));
    }
    assert_eq!(findings, run(".no-mistakes.yml"));
}
#[test]
fn unknowns_limits_and_policy_never_become_assertion_proof() {
    let root = fixture();
    for name in [
        "ignore.yml",
        "depth.yml",
        "nodes.yml",
        "mixed.yml",
        "applications.yml",
        "unrelated-contract.yml",
        "parse-error.yml",
    ] {
        let findings = run(name);
        if name == "ignore.yml" {
            assert!(findings.is_empty(), "{findings:#?}");
        } else {
            assert!(!findings.is_empty(), "{name}: {findings:#?}");
        }
    }
    let options = crate::napi_api::options::test_json_arg(
        serde_json::json!({"root": root, "config": root.join("advise.yml")}),
    );
    let report: serde_json::Value =
        serde_json::from_str(&crate::napi_api::check_json_impl(options).unwrap()).unwrap();
    assert!(report["rules"].as_array().unwrap().is_empty(), "{report}");
    assert!(
        !report["advisories"].as_array().unwrap().is_empty(),
        "{report}"
    );
}
#[test]
fn invalid_configuration_is_actionable() {
    let root = fixture();
    for name in [
        "empty-producers.yml",
        "empty-assertions.yml",
        "invalid-bound.yml",
        "invalid-depth.yml",
        "missing-function.yml",
        "missing-file.yml",
        "invalid-selector.yml",
        "invalid-import.yml",
        "typo.yml",
        "invalid-contract-producer.yml",
        "invalid-contract-owner.yml",
        "invalid-policy.yml",
        "invalid-glob.yml",
    ] {
        let error =
            crate::codebase::rules::run_check(&root, Some(&root.join(name)), None).unwrap_err();
        assert!(
            format!("{error:#}").contains(RULE_ID)
                || (name == "invalid-glob.yml"
                    && format!("{error:#}").contains("rules[0].include")),
            "{name}: {error:#}"
        );
    }
}
#[cfg(feature = "test-instrumentation")]
#[test]
fn existing_graph_projection_is_unchanged_when_the_rule_is_enabled() {
    let root = fixture();
    let query = |config: &str, relationships: &[&str]| {
        let input = serde_json::json!({"root":root,"config":root.join(config),"reports":[
            {"type":"dependents","files":["src/production.mts"],"relationships":relationships},
            {"type":"check"}
        ]});
        let result: serde_json::Value =
            serde_json::from_str(&crate::napi_api::analyze_project_json_impl(input).unwrap())
                .unwrap();
        result["reports"][0]["result"].clone()
    };
    for relationships in [&["call", "import"][..], &["call"][..], &[][..]] {
        assert_eq!(
            query("disabled.yml", relationships),
            query(".no-mistakes.yml", relationships)
        );
    }
}

#[test]
fn all_standard_suppressions_and_custom_messages_work() {
    assert!(run("suppression.yml").is_empty());
    assert!(run("filtered.yml").is_empty());
    assert!(run("custom.yml")
        .iter()
        .all(|finding| finding.message.starts_with("Assert the worker result")));
}
include!("tests/eligibility.rs");

#[cfg(feature = "test-instrumentation")]
mod prepared;

#[test]
fn mixed_known_unknown_assertion_target_cannot_pass() {
    let mut findings = run("mixed.yml");
    findings.extend(run("ambiguous-callback.yml"));
    for symbol in [
        "mixed",
        "mixedTarget",
        "ambiguousCallbackA",
        "ambiguousCallbackB",
    ] {
        assert!(
            findings
                .iter()
                .any(|finding| finding.target.as_deref() == Some(symbol)
                    && finding.message.contains("unanalyzable")),
            "{findings:#?}"
        );
    }
}

#[cfg(feature = "test-instrumentation")]
mod source_failures;

mod public_query;
