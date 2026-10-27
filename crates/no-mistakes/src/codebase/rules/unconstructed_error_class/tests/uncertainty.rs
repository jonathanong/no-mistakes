use super::*;

// Output cases moved to cli_rra_snapshot_unconstructed_error_class.

fn uncertain_fixture() -> PathBuf {
    fixture().parent().unwrap().join("uncertain/fixture")
}

fn report(config: &str, audit: bool) -> serde_json::Value {
    let root = uncertain_fixture();
    let result = crate::napi_api::check_json_impl(crate::napi_api::options::test_json_arg(
        serde_json::json!({ "root": root, "config": root.join(config), "includeSuppressed": audit })
            .to_string(),
    )).unwrap();
    crate::test_support::without_hygiene_advisory_json(serde_json::from_str(&result).unwrap())
}

#[test]
fn uncertain_library_rules_match_the_public_report() {
    let root = uncertain_fixture();
    let baseline = report(".no-mistakes.yml", false);
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
