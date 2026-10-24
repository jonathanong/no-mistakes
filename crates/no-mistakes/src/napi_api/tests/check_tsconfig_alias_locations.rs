use super::*;

#[test]
fn tsconfig_alias_folder_mapping_check_and_project_preserve_entry_locations() {
    for (name, file, lines) in [
        ("valid-files", "tsconfig.json", vec![]),
        (
            "wrong-files",
            "tsconfig.json",
            vec![4, 7, 10, 13, 16, 16, 16, 21, 24],
        ),
        ("locations", "config/policy.base.json", vec![10, 11, 12]),
        ("entry-suppression", "tsconfig.json", vec![6]),
    ] {
        let root = crate::codebase::ts_resolver::normalize_path(
            &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../test-cases/rules/tsconfig-alias-folder-mapping/fixture")
                .join(name),
        );
        let report: serde_json::Value = serde_json::from_str(
            &check_json_impl(crate::napi_api::options::test_json_arg(
                json!({"root": root}).to_string(),
            ))
            .unwrap(),
        )
        .unwrap();
        let findings = report["rules"].as_array().unwrap();
        assert_eq!(findings.len(), lines.len(), "{name}: {report}");
        assert!(
            findings.iter().all(|finding| {
                finding["rule"] == "tsconfig-alias-folder-mapping" && finding["file"] == file
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
        assert_eq!(report["warnings"], json!([]), "{name}: {report}");
        let project: serde_json::Value = serde_json::from_str(
            &crate::napi_api::analyze_project::analyze_project_json_impl(
                crate::napi_api::options::test_json_arg(
                    json!({"root": root, "reports": [{"type": "check"}]}).to_string(),
                ),
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(project["reports"][0]["result"], report, "{name}");
        if name == "entry-suppression" {
            let audit: serde_json::Value = serde_json::from_str(
                &check_json_impl(crate::napi_api::options::test_json_arg(
                    json!({"root": root, "includeSuppressed": true}).to_string(),
                ))
                .unwrap(),
            )
            .unwrap();
            assert_eq!(audit["rules"], report["rules"]);
            let suppressed = audit["suppressed"].as_array().unwrap();
            assert_eq!(suppressed.len(), 1, "{audit}");
            assert_eq!(suppressed[0]["line"], 5);
            assert_eq!(suppressed[0]["directive"]["kind"], "nextLine");
        }
    }
}
