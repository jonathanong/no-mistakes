use super::*;

#[test]
fn generator_creation_and_unsupported_consumption_abstain_under_every_policy() {
    let root = fixture();
    for (policy, expected_findings, expected_advisories) in
        [("report", 7, 0), ("advise", 0, 7), ("ignore", 0, 0)]
    {
        let input = crate::napi_api::options::test_json_arg(serde_json::json!({
            "root": root, "config": root.join(format!("generators-{policy}.yml"))
        }));
        let report: serde_json::Value = crate::test_support::without_hygiene_advisory_json(
            serde_json::from_str(&crate::napi_api::check_json_impl(input).unwrap()).unwrap(),
        );
        assert_eq!(
            report["rules"].as_array().unwrap().len(),
            expected_findings,
            "{report}"
        );
        assert_eq!(
            report["advisories"].as_array().unwrap().len(),
            expected_advisories,
            "{report}"
        );
        for finding in report["rules"]
            .as_array()
            .unwrap()
            .iter()
            .chain(report["advisories"].as_array().unwrap())
        {
            assert!(
                finding["message"]
                    .as_str()
                    .unwrap()
                    .contains("unanalyzable"),
                "{report}"
            );
        }
    }
}

// The graph JSON test facade is exported only with instrumentation.
#[cfg(feature = "test-instrumentation")]
#[test]
fn generator_contracts_do_not_emit_executable_ownership_edges() {
    let root = fixture();
    let input = serde_json::json!({
        "root": root, "config": root.join("generators-report.yml"),
        "reports": [{ "type": "dependencies", "includeSymbols": true,
            "files": ["src/producers.mts#generatorContract", "src/producers.mts#generatorContractOwner"],
            "relationships": ["assertion-owner"] }]
    });
    let report: serde_json::Value = crate::test_support::without_hygiene_advisory_json(
        serde_json::from_str(&crate::napi_api::analyze_project_json_impl(input).unwrap()).unwrap(),
    );
    assert_eq!(
        report["reports"][0]["result"]["files"],
        serde_json::json!([]),
        "{report}"
    );
}

#[test]
fn import_forms_and_external_calls_preserve_uncertainty_policy() {
    let root = fixture();
    for (policy, channel) in [
        ("report", "rules"),
        ("advise", "advisories"),
        ("ignore", "rules"),
    ] {
        let input = crate::napi_api::options::test_json_arg(serde_json::json!({
            "root": root, "config": root.join(format!("import-forms-{policy}.yml"))
        }));
        let report: serde_json::Value = crate::test_support::without_hygiene_advisory_json(
            serde_json::from_str(&crate::napi_api::check_json_impl(input).unwrap()).unwrap(),
        );
        let findings = report[channel].as_array().unwrap();
        let mut targets = findings
            .iter()
            .map(|finding| {
                assert!(
                    finding["message"]
                        .as_str()
                        .unwrap()
                        .contains("unanalyzable"),
                    "{report}"
                );
                finding["target"].as_str().unwrap()
            })
            .collect::<Vec<_>>();
        targets.sort_unstable();
        let expected = if policy == "ignore" {
            vec![]
        } else {
            vec!["defaultMemberAssertion", "externalAndAssertion"]
        };
        assert_eq!(targets, expected, "{report}");
        let other = if channel == "rules" {
            "advisories"
        } else {
            "rules"
        };
        assert!(report[other].as_array().unwrap().is_empty(), "{report}");
        assert!(
            report["warnings"].as_array().unwrap().is_empty(),
            "{report}"
        );
    }
}

#[test]
fn callback_style_fs_treats_nonliteral_last_arguments_as_possible_callbacks() {
    let root = crate::test_support::rule_fixture_root("coverage-producer-assertion-owner-fs");
    let findings =
        crate::codebase::rules::run_check(&root, Some(&root.join(".no-mistakes.yml")), None)
            .unwrap();
    for symbol in [
        "ternaryCallback",
        "logicalCallback",
        "computedCallback",
        "dynamicTemplateArgument",
        "spreadArguments",
        "applyCallbackArgument",
        "applySpreadArguments",
        "callCallbackArgument",
        "ternaryOptions",
        "fnPropertyOptions",
        "functionPropertyOptions",
        "classPropertyOptions",
        "nestedFnPropertyOptions",
        "unknownPropertyOptions",
        "memberPropertyOptions",
        "callPropertyOptions",
        "conditionalPropertyOptions",
        "logicalPropertyOptions",
        "arrayFunctionOptions",
        "arrayUnknownOptions",
        "methodOptions",
        "getterOptions",
        "shorthandOptions",
        "spreadOptions",
        "arraySpreadOptions",
        "arrayHoleOptions",
        "computedKeyOptions",
        "computedStaticKeyOptions",
        "templatePropertyOptions",
        "callFnPropertyOptions",
        "applyFnPropertyOptions",
        "depthOptions",
    ] {
        assert!(
            findings.iter().any(|finding| finding.rule == RULE_ID
                && finding.target.as_deref() == Some(symbol)
                && finding.message.contains("unanalyzable")),
            "{symbol}: {findings:#?}"
        );
    }
    for symbol in [
        "streamOptions",
        "nestedDataOptions",
        "nearDepthOptions",
        "mkdirOptions",
        "typedStreamOptions",
        "arrayOptions",
        "callObjectOptions",
        "applyObjectOptions",
        "applyArrayOptions",
    ] {
        assert!(
            !findings.iter().any(|finding| finding.rule == RULE_ID
                && finding.target.as_deref() == Some(symbol)),
            "{symbol}: {findings:#?}"
        );
    }
    for symbol in [
        "literalLastArgument",
        "templateLiteralArgument",
        "typedLiteralArgument",
        "noArguments",
        "applyLiteralArgument",
        "applyTemplateLiteralArgument",
        "applyNoArguments",
        "applyNullArguments",
        "callLiteralArgument",
    ] {
        assert!(
            findings.iter().any(|finding| finding.rule == RULE_ID
                && finding.target.as_deref() == Some(symbol)
                && !finding.message.contains("unanalyzable")),
            "{findings:#?}"
        );
    }
}

#[test]
fn literal_switches_only_count_executable_assertion_cases() {
    let findings = run("switches.yml");
    let mut targets = findings
        .iter()
        .map(|finding| finding.target.as_deref().unwrap())
        .collect::<Vec<_>>();
    targets.sort_unstable();
    assert_eq!(
        targets,
        vec![
            "blockBreak",
            "blockReturn",
            "defaultBeforeMatch",
            "emptySwitch",
            "impossibleCase",
            "loneSurrogateWrongCase",
            "returnedCase",
            "stoppedFallthrough",
            "thrownCase",
            "typedCaseMismatch"
        ],
        "{findings:#?}"
    );
}
