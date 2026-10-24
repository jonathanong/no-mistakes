use super::*;

fn root(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/no-mistakes-config/fixture")
            .join(name),
    )
}

fn options(root: &std::path::Path, config: &str) -> serde_json::Value {
    json!({"root": root, "config": root.join(config)})
}

fn check(input: serde_json::Value) -> serde_json::Value {
    serde_json::from_str(
        &check_json_impl(crate::napi_api::options::test_json_arg(input.to_string())).unwrap(),
    )
    .unwrap()
}

#[test]
fn config_locations_check_and_project_keep_authored_entries_and_rule_names() {
    for (name, file, lines) in [
        ("locations", ".no-mistakes.yml", vec![4, 11, 12, 17, 19]),
        (
            "locations-shapes",
            ".no-mistakes.yml",
            vec![4, 5, 7, 10, 13, 19, 25, 26],
        ),
        ("locations-targeted", ".no-mistakes.yml", vec![8]),
        ("locations-custom", "policy.yml", vec![7]),
        ("locations-json", "policy.json", vec![8]),
        ("locations-jsonc", "policy.jsonc", vec![10, 12]),
        ("locations-suppression", ".no-mistakes.yml", vec![9]),
    ] {
        let root = root(name);
        let mut input = options(&root, file);
        let report = check(input.clone());
        let findings = report["rules"].as_array().unwrap();
        assert_eq!(findings.len(), lines.len(), "{name}: {report}");
        assert!(
            findings.iter().all(|finding| {
                finding["rule"] == "no-mistakes-config" && finding["file"] == file
            }),
            "{name}: {report}"
        );
        assert_eq!(
            findings
                .iter()
                .map(|finding| finding["line"].as_u64().unwrap())
                .collect::<Vec<_>>(),
            lines,
            "{name}: {report}",
        );
        for finding in findings {
            let message = finding["message"].as_str().unwrap();
            if message.starts_with("rules[") {
                assert!(
                    message.contains(" (unique-exports)")
                        || message.contains(" (forbidden-dependencies)"),
                    "{name}: {message}"
                );
            }
        }
        input["reports"] = json!([{"type": "check"}]);
        let project: serde_json::Value = serde_json::from_str(
            &crate::napi_api::analyze_project::analyze_project_json_impl(
                crate::napi_api::options::test_json_arg(input.to_string()),
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(project["reports"][0]["result"], report, "{name}");
    }
}

#[test]
fn config_locations_suppression_audit_keeps_the_other_entry_visible() {
    let root = root("locations-suppression");
    let mut input = options(&root, ".no-mistakes.yml");
    let baseline = check(input.clone());
    input["includeSuppressed"] = json!(true);
    let audit = check(input);
    assert_eq!(audit["rules"], baseline["rules"]);
    let suppressed = audit["suppressed"].as_array().unwrap();
    assert_eq!(suppressed.len(), 1, "{audit}");
    assert_eq!(suppressed[0]["line"], 8);
    assert_eq!(suppressed[0]["directive"]["line"], 7);
    assert_eq!(suppressed[0]["directive"]["kind"], "nextLine");
}

#[test]
fn config_locations_fatal_globs_keep_the_same_public_check_and_project_context() {
    for (name, line, field) in [
        (
            "locations-invalid-exclude",
            8,
            "rules[1] (unique-exports).exclude[1]",
        ),
        (
            "locations-invalid-include",
            11,
            "rules[2] (forbidden-dependencies).include[1]",
        ),
        (
            "locations-invalid-project",
            5,
            "projects.app.with[brackets].include[1]",
        ),
    ] {
        let root = root(name);
        let mut input = options(&root, ".no-mistakes.yml");
        let expected = format!(".no-mistakes.yml:{line}: {field}: contains invalid glob `[bad`");
        let error = check_json_impl(crate::napi_api::options::test_json_arg(input.to_string()))
            .unwrap_err()
            .to_string();
        assert!(error.contains(&expected), "{name}: {error}");
        input["reports"] = json!([{"type": "check"}]);
        let error = crate::napi_api::analyze_project::analyze_project_json_impl(
            crate::napi_api::options::test_json_arg(input.to_string()),
        )
        .unwrap_err()
        .to_string();
        assert!(error.contains(&expected), "{name}: {error}");
    }
}
