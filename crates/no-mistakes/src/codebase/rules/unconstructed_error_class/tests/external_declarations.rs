use super::*;

// Output cases moved to cli_rra_snapshot_unconstructed_error_class.

#[test]
fn declaration_ancestry_audit_preserves_suppressed_channel() {
    let root = fixture().parent().unwrap().join("declarations/fixture");
    for config in [
        ".no-mistakes.yml",
        "max.yml",
        "low.yml",
        "advise.yml",
        "ignore.yml",
    ] {
        let json = crate::napi_api::check_json_impl(crate::napi_api::options::test_json_arg(
            serde_json::json!({"root":root,"config":root.join(config),"includeSuppressed":true})
                .to_string(),
        ))
        .unwrap();
        let report: serde_json::Value = crate::test_support::without_hygiene_advisory_json(
            serde_json::from_str(&json).unwrap(),
        );
        assert_eq!(
            report["suppressed"].as_array().unwrap().len(),
            if config == "ignore.yml" { 0 } else { 1 },
            "{config}: {report}"
        );
    }
}
