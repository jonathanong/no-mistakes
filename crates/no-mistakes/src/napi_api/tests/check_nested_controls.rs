use super::*;
use serde_json::{json, Value};

#[test]
fn check_json_reports_native_control_nesting() {
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/react-no-nested-buttons-and-links");
    let output = check_json_impl(crate::napi_api::options::test_json_arg(
        json!({
            "root": fixture.join("fixture"),
            "config": fixture.join("configs/check.yml"),
            "includeSuppressed": true,
        })
        .to_string(),
    ))
    .unwrap();
    let report: Value = serde_json::from_str(&output).unwrap();
    let findings = report["rules"].as_array().unwrap();
    assert_eq!(findings.len(), 4, "{report}");
    assert!(
        findings.iter().any(|finding| {
            finding["rule"] == "react-no-nested-buttons-and-links"
                && finding["file"] == "actions.tsx"
                && finding["line"] == 5
        }),
        "{report}"
    );
    let batch: Value = serde_json::from_str(
        &crate::napi_api::analyze_project::analyze_project_json_impl(
            crate::napi_api::options::test_json_arg(
                json!({
                    "root": fixture.join("fixture"),
                    "config": fixture.join("configs/check.yml"),
                    "reports": [{ "type": "check", "includeSuppressed": true }],
                })
                .to_string(),
            ),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(batch["reports"][0]["result"], report);
    let suppressed = report["suppressed"].as_array().unwrap();
    assert_eq!(suppressed.len(), 3, "{report}");
}

#[test]
fn rust_checks_native_control_nesting_from_supplied_files() {
    use crate::codebase::rules::{nested_controls, run_filesystem_rules_with_config};

    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/react-no-nested-buttons-and-links");
    let root = crate::codebase::ts_resolver::normalize_path(&fixture.join("fixture"));
    let config =
        crate::config::v2::load_v2_config(&root, Some(&fixture.join("configs/check.yml"))).unwrap();
    let files = crate::codebase::ts_source::discover_files(&root, &[]);
    let direct = nested_controls::check_with_files(&root, &config, &files).unwrap();
    let dispatched = run_filesystem_rules_with_config(&root, &config, &files).unwrap();
    assert_eq!(direct.len(), 4);
    assert_eq!(direct, dispatched);
}
