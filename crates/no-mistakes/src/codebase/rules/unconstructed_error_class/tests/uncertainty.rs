use super::*;

fn uncertain_fixture() -> PathBuf {
    fixture().parent().unwrap().join("uncertain/fixture")
}

fn report(config: &str, audit: bool) -> serde_json::Value {
    let root = uncertain_fixture();
    let result = crate::napi_api::check_json_impl(crate::napi_api::options::test_json_arg(
        serde_json::json!({ "root": root, "config": root.join(config), "includeSuppressed": audit })
            .to_string(),
    )).unwrap();
    serde_json::from_str(&result).unwrap()
}

#[test]
fn uncertain_inputs_route_through_every_shared_policy_and_supported_effort() {
    let baseline = report(".no-mistakes.yml", false);
    assert_eq!(baseline["warnings"], serde_json::json!([]));
    for (config, failures, advisories) in [
        (".no-mistakes.yml", 8, 0),
        ("max.yml", 8, 0),
        ("low.yml", 8, 0),
        ("advise.yml", 1, 7),
        ("ignore.yml", 1, 0),
        ("repo-advise.yml", 1, 7),
        ("override.yml", 8, 0),
    ] {
        let value = report(config, false);
        assert_eq!(
            value["rules"].as_array().unwrap().len(),
            failures,
            "{config}: {value}"
        );
        assert_eq!(
            value["advisories"].as_array().unwrap().len(),
            advisories,
            "{config}: {value}"
        );
        if matches!(config, "max.yml" | "low.yml") {
            assert_eq!(value["rules"], baseline["rules"], "{config}");
        }
        if config == "low.yml" {
            assert!(value["warnings"][0]
                .as_str()
                .unwrap()
                .contains("effective effort max"));
        }
    }
    let rules = baseline["rules"].as_array().unwrap();
    let definite: Vec<_> = rules
        .iter()
        .filter(|finding| finding["target"] != "unanalyzable")
        .map(|finding| finding["target"].as_str().unwrap())
        .collect();
    // DynamicError.make(DynamicError) really constructs DynamicError: unresolved
    // factories cannot justify a definite unused claim. Unrelated dead code still can.
    assert_eq!(definite, ["DefinitelyDead"]);
    for (file, line, cause) in [
        ("src/unknown.ts", 2, "unresolved constructor occurs"),
        ("src/unknown.ts", 6, "unresolved constructor occurs"),
        ("src/unknown.ts", 3, "Cannot resolve construction target"),
        ("src/unknown.ts", 7, "Cannot resolve construction target"),
        ("src/namespace.ts", 2, "namespace escapes"),
        ("src/external.ts", 2, "base is outside"),
        ("src/external.ts", 5, "base is outside"),
    ] {
        assert!(
            rules.iter().any(|finding| finding["file"] == file
                && finding["line"] == line
                && finding["target"] == "unanalyzable"
                && finding["message"].as_str().unwrap().contains(cause)),
            "{file}:{line}: {baseline}"
        );
    }
    let root = uncertain_fixture();
    assert_eq!(
        serde_json::to_value(run_check(&root, None, None).unwrap()).unwrap(),
        baseline["rules"]
    );
}

#[test]
fn uncertainty_honors_line_file_suppression_and_audit_for_both_channels() {
    for config in [".no-mistakes.yml", "advise.yml", "ignore.yml"] {
        let baseline = report(config, false);
        let audit = report(config, true);
        assert_eq!(audit["rules"], baseline["rules"]);
        assert_eq!(audit["advisories"], baseline["advisories"]);
        let suppressed = audit["suppressed"].as_array().unwrap();
        assert_eq!(
            suppressed.len(),
            if config == "ignore.yml" { 0 } else { 8 },
            "{audit}"
        );
        assert!(suppressed
            .iter()
            .all(|finding| finding["reason"].as_str().unwrap().contains("Cannot")));
    }
}

#[test]
fn custom_messages_preserve_uncertainty_causes_and_rule_scope_is_authoritative() {
    let custom = report("custom.yml", false);
    assert_eq!(custom["rules"].as_array().unwrap().len(), 8, "{custom}");
    assert!(custom["rules"].as_array().unwrap().iter().all(|finding| {
        finding["message"]
            .as_str()
            .unwrap()
            .starts_with("Inspect error construction:")
    }));
    let included = report("include.yml", false);
    assert_eq!(included["rules"].as_array().unwrap().len(), 4, "{included}");
    assert!(included["rules"]
        .as_array()
        .unwrap()
        .iter()
        .all(|finding| finding["file"] == "src/unknown.ts"));
}

#[test]
fn static_this_factories_remain_uncertain_rather_than_definitely_unconstructed() {
    let root = fixture();
    let all = run_check(&root, None, None).unwrap();
    for name in [
        "ThisFactoryError",
        "NestedFactoryError",
        "Errors.TopicError",
        "Errors.Inner.DeepError",
    ] {
        assert!(
            all.iter()
                .any(|finding| finding.target.as_deref() == Some("unanalyzable")
                    && finding.message.contains(&format!("`{name}`"))
                    && finding.message.contains("unresolved constructor occurs")),
            "{name}: {all:#?}"
        );
        assert!(
            !all.iter()
                .any(|finding| finding.target.as_deref() == Some(name)),
            "{name}"
        );
    }
}
