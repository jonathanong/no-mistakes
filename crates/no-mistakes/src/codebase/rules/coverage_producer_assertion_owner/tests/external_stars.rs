#[test]
fn plain_external_stars_preserve_identity_without_losing_findings() {
    let root =
        crate::test_support::rule_fixture_root("coverage-producer-assertion-owner-external-stars");
    let findings = crate::codebase::rules::run_check(&root, None, None).unwrap();
    assert_eq!(findings.len(), 12, "{findings:#?}");
    for (symbol, detail) in [
        ("plainStar", "external call has no ownership model"),
        ("chainedStar", "external call has no ownership model"),
        ("localExportShadow", "callee target is unknown"),
        ("localParameterShadow", "callee target is unknown"),
        ("competingStars", "callee target is unknown"),
        ("competingLocalStar", "callee target is unknown"),
        ("cycle", "callee target is unknown"),
        ("missingSource", "callee target is unknown"),
        ("defaultImport", "callee target is unknown"),
        ("dottedMember", "external call has no ownership model"),
        ("fsStar", "has no reachable assertion owner"),
        ("fsDirect", "has no reachable assertion owner"),
    ] {
        let finding = findings
            .iter()
            .find(|finding| finding.target.as_deref() == Some(symbol))
            .unwrap_or_else(|| panic!("missing {symbol}: {findings:#?}"));
        assert!(finding.message.contains(detail), "{symbol}: {finding:#?}");
    }
    assert!(!findings
        .iter()
        .any(|finding| finding.target.as_deref() == Some("assertionStar")));
}

#[test]
fn native_check_resolves_plain_external_stars_and_keeps_controls_unknown() {
    let root =
        crate::test_support::rule_fixture_root("coverage-producer-assertion-owner-external-stars");
    for (config, count) in [(".no-mistakes.yml", 12), ("disabled.yml", 0)] {
        let input = crate::napi_api::options::test_json_arg(serde_json::json!({
            "root": root, "config": root.join(config)
        }));
        let report: serde_json::Value =
            serde_json::from_str(&crate::napi_api::check_json_impl(input).unwrap()).unwrap();
        let findings = report["rules"].as_array().unwrap();
        assert_eq!(findings.len(), count, "{report}");
        if count > 0 {
            for (symbol, detail) in [
                ("plainStar", "external call has no ownership model"),
                ("cycle", "callee target is unknown"),
                ("fsStar", "has no reachable assertion owner"),
            ] {
                let finding = findings
                    .iter()
                    .find(|finding| finding["target"] == symbol)
                    .unwrap();
                assert!(
                    finding["message"].as_str().unwrap().contains(detail),
                    "{finding}"
                );
            }
        }
    }
}
