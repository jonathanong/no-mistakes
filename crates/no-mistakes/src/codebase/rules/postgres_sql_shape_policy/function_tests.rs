use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};

// Output cases moved to cli_postgres_snapshot_sql_shape_policy.

pub(super) fn fixture() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-sql-shape-policy/fixture/banned-functions"),
    )
}

fn config(names: &str) -> NoMistakesConfig {
    NoMistakesConfig { rules: vec![RuleDef { rule: RULE_ID.into(), scope: Some(RuleScope::Repository), options: serde_yaml::from_str(&format!("bannedShapes: [banned-function-call]\nshapeOptions:\n  bannedFunctionCall:\n    functions: [{names}]\nimportSpecifier: '@example/db'\ntrustedSqlTags: [{{module: '@example/db', name: sql}}]\nunanalyzableSql: ignore")).unwrap(), ..Default::default() }], ..Default::default() }
}

#[test]
fn empty_and_blank_function_names_are_configuration_errors_only_when_enabled() {
    for names in ["", "' '", "'pg_catalog.'"] {
        let cfg = config(names);
        let opts: Options = cfg.rules[0].try_rule_options().unwrap();
        assert!(compile_options(&opts).is_err(), "{names}");
    }
    assert!(compile_options(&Options::default()).is_ok());
}

#[test]
fn dynamic_calls_use_the_new_shape_target_and_suppression_is_consistent() {
    let root = fixture();
    let mut cfg = config("pg_sleep");
    cfg.rules[0].options["unanalyzableSql"] = serde_yaml::Value::String("fail".into());
    let files = [root.join("disabled.sql"), root.join("disabled-file.sql")];
    let mut findings = check_with_files(&root, &cfg, &files).unwrap();
    assert_eq!(findings.len(), 3);
    let sources = super::super::source_store_for_files(&files);
    super::super::suppress_rule_findings_with_sources(&root, &mut findings, &sources);
    assert!(findings.is_empty());
}
