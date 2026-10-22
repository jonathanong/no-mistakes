use super::*;

#[test]
fn async_check_options_preserve_sql_alias_and_default_effort_reports() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/rule-analysis-policy/fixture");
    for mode in ["report", "advise", "ignore"] {
        let execute = |config: String, effort: Option<&str>| {
            let mut options = serde_json::json!({"root": root, "config": config});
            if let Some(effort) = effort {
                options["effort"] = effort.into();
            }
            serde_json::from_str::<serde_json::Value>(
                &check_json_impl(crate::napi_api::options::test_json_arg(options.to_string()))
                    .unwrap(),
            )
            .unwrap()
        };
        let default = execute(format!("sql-{mode}.yml"), None);
        assert_eq!(execute(format!("legacy-sql-{mode}.yml"), None), default);
        assert_eq!(execute(format!("sql-{mode}.yml"), Some("max")), default);
        let low = execute(format!("sql-{mode}.yml"), Some("low"));
        assert_eq!(execute(format!("legacy-sql-{mode}.yml"), Some("low")), low);
        // Low deliberately replaces deep inference with static rewrite guidance.
        // Policy routing and finding identity must still match the max tier.
        for field in ["rules", "advisories"] {
            let rows = low[field].as_array().unwrap();
            let expected = default[field].as_array().unwrap();
            assert_eq!(rows.len(), expected.len());
            for (actual, expected) in rows.iter().zip(expected) {
                for key in ["rule", "file", "line"] {
                    assert_eq!(actual[key], expected[key]);
                }
                let message = actual["message"].as_str().unwrap();
                assert!(message.contains("query(\"/* name */ \" + sql)"));
                assert!(message.contains("one local const"));
            }
        }
    }
}
