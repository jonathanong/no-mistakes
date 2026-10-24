use super::*;

#[test]
fn unconstructed_error_check_and_project_reports_preserve_both_channels_and_audit() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/unconstructed-error-class/uncertain/fixture");
    for config in [".no-mistakes.yml", "max.yml", "advise.yml", "ignore.yml"] {
        let options =
            json!({ "root": root, "config": root.join(config), "includeSuppressed": true });
        let check: serde_json::Value = serde_json::from_str(
            &check_json_impl(crate::napi_api::options::test_json_arg(options.to_string())).unwrap(),
        )
        .unwrap();
        let project: serde_json::Value = serde_json::from_str(
            &crate::napi_api::analyze_project::analyze_project_json_impl(
                crate::napi_api::options::test_json_arg(
                    json!({
                        "root": root, "config": root.join(config),
                        "reports": [{ "type": "check", "includeSuppressed": true }],
                    })
                    .to_string(),
                ),
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(
            project["reports"][0]["result"], check,
            "{config}: {project}"
        );
    }
}
