use super::*;

#[test]
fn unconstructed_error_check_and_project_reports_preserve_both_channels_and_audit() {
    for fixture in ["uncertain", "declarations"] {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!(
            "../../test-cases/rules/unconstructed-error-class/{fixture}/fixture"
        ));
        for config in [".no-mistakes.yml", "max.yml", "advise.yml", "ignore.yml"] {
            let options =
                json!({ "root": root, "config": root.join(config), "includeSuppressed": true });
            let check: serde_json::Value = serde_json::from_str(
                &check_json_impl(crate::napi_api::options::test_json_arg(options.to_string()))
                    .unwrap(),
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
}

#[test]
fn unconstructed_error_check_does_not_widen_ordinary_project_queries() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/unconstructed-error-class/declarations/fixture");
    let analyze = |include_check| {
        let mut reports = vec![
            json!({"type":"importers","file":"src/classes.ts"}),
            json!({"type":"callSites","file":"src/classes.ts","exportName":"render"}),
        ];
        if include_check {
            reports.push(json!({"type":"check"}));
        }
        let output = crate::napi_api::analyze_project::analyze_project_json_impl(
            crate::napi_api::options::test_json_arg(
                json!({"root":root,"reports":reports}).to_string(),
            ),
        )
        .unwrap();
        serde_json::from_str::<serde_json::Value>(&output).unwrap()
    };
    let ordinary = analyze(false);
    let combined = analyze(true);
    for index in 0..2 {
        assert_eq!(ordinary["reports"][index], combined["reports"][index]);
    }
    assert!(ordinary["reports"].to_string().contains("consumer.ts"));
}
