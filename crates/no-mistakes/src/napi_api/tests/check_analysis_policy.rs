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
        let mut low = execute(format!("sql-{mode}.yml"), Some("low"));
        assert_eq!(low["warnings"], serde_json::json!([
            "postgres-require-query-annotation: requested effort low; effective effort max because this rule has no low implementation"
        ]));
        low["warnings"] = serde_json::json!([]);
        assert_eq!(low, default);
    }
}
