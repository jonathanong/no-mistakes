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

#[test]
fn followup_probes_require_proven_uuid_columns_at_the_catalog_boundary() {
    let root = fixture();
    let files = vec![root.join("probe-followup-schema.json")];
    let sources = super::super::source_store_for_files(&files);
    let catalog = crate::codebase::postgres::SchemaCatalog::load(
        &root,
        "probe-followup-schema.json",
        &sources,
    )
    .unwrap();
    let columns = super::evaluate::Columns::new(&catalog);
    for kind in ["view", "distinct", "null"] {
        for status in ["pass", "positive"] {
            let name = format!("probe-followup-{kind}-{status}.sql");
            let source = std::fs::read_to_string(root.join(&name)).unwrap();
            let facts = crate::codebase::postgres::extract_sql_statement_facts(&source);
            assert!(!facts.parse_failed, "{name}");
            let mut reported = std::collections::BTreeSet::new();
            for fact in &facts.transformed_column_uses {
                if !matches!(columns.resolve(fact), Ok((_, false))) {
                    reported.insert(fact.line);
                }
            }
            if status == "pass" {
                assert!(reported.is_empty(), "{name}: {reported:?}");
            } else {
                let expected: std::collections::BTreeSet<_> =
                    (1..=source.lines().count()).collect();
                assert_eq!(reported, expected, "{name}");
            }
        }
    }
}
