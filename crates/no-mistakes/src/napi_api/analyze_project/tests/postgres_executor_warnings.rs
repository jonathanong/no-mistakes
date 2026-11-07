use super::*;

#[test]
fn analyze_project_preserves_executor_warning_parity_with_standalone_check() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/check/postgres-executor-names"),
    );
    for (config, warning_key) in [
        ("warning-only.yml", Some("executorNames")),
        ("exclude-unused.yml", Some("excludeExecutorNames")),
        ("exclude-unimported.yml", Some("excludeExecutorNames")),
        ("exclude-defaults.yml", None),
        ("null-import.yml", None),
    ] {
        let options = json!({ "root": root, "config": root.join(config) });
        let expected: Value = serde_json::from_str(
            &crate::napi_api::check_json_impl(crate::napi_api::options::test_json_arg(
                options.to_string(),
            ))
            .unwrap(),
        )
        .unwrap();
        let mut options = options;
        options["reports"] = json!([{ "type": "check" }]);
        let project: Value =
            serde_json::from_str(&analyze_project_value_impl(options).unwrap()).unwrap();
        let report = &project["reports"][0]["result"];
        assert_eq!(report, &expected, "{config}: {project}");
        if let Some(key) = warning_key {
            assert!(
                report["warnings"].to_string().contains(key),
                "{config}: {report}"
            );
        } else {
            assert_eq!(report["warnings"], json!([]), "{config}: {report}");
        }
    }
}
