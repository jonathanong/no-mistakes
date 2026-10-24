use super::*;

#[test]
fn transformed_uuid_aggregate_values_and_comparison_operands_match_public_apis() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/check-runner/postgres-uuid-predicate-operands/fixture"),
    );
    let mut previous = None;
    for effort in ["low", "max"] {
        let input = json!({
            "root": root,
            "config": root.join(".no-mistakes.aggregate-operands.yml"),
            "effort": effort,
        });
        let check: serde_json::Value = serde_json::from_str(
            &check_json_impl(crate::napi_api::options::test_json_arg(input.to_string())).unwrap(),
        )
        .unwrap();
        assert_eq!(check["warnings"], json!([]), "{effort}: {check}");
        assert_eq!(check["advisories"], json!([]), "{effort}: {check}");
        let findings = check["rules"].as_array().unwrap();
        let mut actual = findings
            .iter()
            .map(|finding| {
                assert_eq!(finding["rule"], "postgres-no-transformed-uuid-predicates");
                assert!(finding["message"]
                    .as_str()
                    .unwrap()
                    .contains("Cast the parameter to uuid"));
                (
                    finding["file"].as_str().unwrap(),
                    finding["line"].as_u64().unwrap(),
                    finding["target"].as_str().unwrap(),
                )
            })
            .collect::<Vec<_>>();
        actual.sort();
        let expected = [
            ("public-aggregate-operands.sql", 12, "users.id"),
            ("public-aggregate-operands.sql", 13, "users.id"),
            // The other operand genuinely transforms a different UUID column.
            ("public-aggregate-operands.sql", 14, "users.external_ref"),
            ("public-aggregate-operands.sql", 15, "users.id"),
            ("public-aggregate-operands.sql", 16, "users.id"),
            ("public-aggregate-operands.sql", 17, "users.id"),
            ("public-aggregate-operands.ts", 14, "users.id"),
            ("public-aggregate-operands.ts", 15, "users.id"),
            ("public-aggregate-operands.ts", 16, "users.external_ref"),
            ("public-aggregate-operands.ts", 17, "users.id"),
            ("public-aggregate-operands.ts", 18, "users.id"),
            ("public-aggregate-operands.ts", 19, "users.id"),
        ];
        assert_eq!(actual, expected, "{effort}: {check}");
        let project_input = json!({
            "root": root,
            "config": input["config"],
            "reports": [{"type": "check", "effort": effort}],
        });
        let project: serde_json::Value = serde_json::from_str(
            &crate::napi_api::analyze_project::analyze_project_json_impl(
                crate::napi_api::options::test_json_arg(project_input.to_string()),
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(project["reports"][0]["result"], check, "{effort}");
        if let Some(previous) = &previous {
            assert_eq!(&check, previous, "both efforts must keep the same findings");
        }
        previous = Some(check);
    }
}
