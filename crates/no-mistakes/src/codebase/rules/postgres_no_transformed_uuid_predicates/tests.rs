use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
pub(super) fn fixture() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-no-transformed-uuid-predicates/fixture"),
    )
}
pub(super) fn config(extra: &str) -> NoMistakesConfig {
    NoMistakesConfig { rules: vec![RuleDef {
        rule: RULE_ID.into(), scope: Some(RuleScope::Repository),
        options: crate::codebase::postgres::tests::fixture_rule_options(&format!(
            "schemaCatalogPath: schema.json\nsqlInclude: ['*.sql']\nimportSpecifier: db\nexecutorNames: [query]\n{extra}")),
        ..Default::default()
    }], ..Default::default() }
}

// Output cases moved to cli_postgres_snapshot_transformed_uuid_predicates.

#[test]
fn options_require_a_complete_schema_catalog() {
    assert!(compile_options(&Options::default()).is_err());
    let opts = Options {
        schema_catalog_path: "schema.json".into(),
        unanalyzable_sql: "invalid".into(),
        ..Default::default()
    };
    assert!(compile_options(&opts)
        .err()
        .unwrap()
        .to_string()
        .contains("unanalyzableSql"));
}
