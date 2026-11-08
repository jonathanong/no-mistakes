use super::*;

#[test]
fn external_namespace_reexports_keep_provenance_and_stored_callbacks_stay_uncertain() {
    let root = crate::test_support::rule_fixture_root("coverage-producer-assertion-owner-builtins");
    for (config, callee, detail) in [
        (
            "namespace-default",
            "DefaultSdk.captureException",
            "external call has no ownership model",
        ),
        (
            "namespace-named",
            "NamedSdk.captureException",
            "external call has no ownership model",
        ),
        (
            "namespace-export",
            "NamespaceSdk.captureException",
            "external call has no ownership model",
        ),
        (
            "namespace-nested",
            "Barrel.NamedSdk.captureException",
            "external call has no ownership model",
        ),
        (
            "namespace-cycle",
            "CyclicSdk.captureException",
            "external call has no ownership model",
        ),
        (
            "namespace-shadowed",
            "DefaultSdk.captureException",
            "callee target is unknown",
        ),
        (
            "namespace-ambiguous",
            "AmbiguousSdk.captureException",
            "callee target is unknown",
        ),
        ("namespace-bare", "DefaultSdk", "callee target is unknown"),
        (
            "namespace-value",
            "ValueSdk.captureException",
            "external call has no ownership model",
        ),
        ("map-get", "handler", "callee target is unknown"),
        ("array-callback", "check", "callee target is unknown"),
        (
            "builtin-name-only",
            "receiver.get",
            "callee target is unknown",
        ),
    ] {
        let findings = crate::codebase::rules::run_check(
            &root,
            Some(&root.join(format!("{config}.yml"))),
            None,
        )
        .unwrap();
        let findings: Vec<_> = findings
            .iter()
            .filter(|finding| finding.rule == RULE_ID)
            .collect();
        assert_eq!(findings.len(), 1, "{config}: {findings:#?}");
        let message = &findings[0].message;
        assert!(message.contains("unanalyzable"), "{config}: {message}");
        assert!(
            message.contains(&format!("callee `{callee}` ({detail})")),
            "{config}: {message}"
        );
    }
}

#[test]
fn node_check_preserves_namespace_uncertainty_and_disabled_output() {
    let root = crate::test_support::rule_fixture_root("coverage-producer-assertion-owner-builtins");
    let check = |config: &str| {
        let input = crate::napi_api::options::test_json_arg(serde_json::json!({
            "root": root, "config": root.join(config)
        }));
        crate::test_support::without_hygiene_advisory_json(
            serde_json::from_str(&crate::napi_api::check_json_impl(input).unwrap()).unwrap(),
        )
    };
    let report = check("namespace-default.yml");
    assert_eq!(report["rules"].as_array().unwrap().len(), 1, "{report}");
    assert!(
        report["rules"][0]["message"]
            .as_str()
            .unwrap()
            .contains("external call has no ownership model"),
        "{report}"
    );
    let report = check("disabled.yml");
    assert!(report["rules"].as_array().unwrap().is_empty(), "{report}");
    assert!(
        report["warnings"].as_array().unwrap().is_empty(),
        "{report}"
    );
}
