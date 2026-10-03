use super::evaluate::offenders;
use super::{check_with_files, check_with_files_and_sources, compile_options, Options, RULE_ID};
use crate::codebase::postgres::{extract_sql_statement_facts, SchemaCatalog};
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::{Path, PathBuf};

fn catalog() -> SchemaCatalog {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-bounded-statements/fixture");
    let sources = crate::codebase::rules::source_store_for_files(std::slice::from_ref(
        &root.join("schema.json"),
    ));
    SchemaCatalog::load(&root, "schema.json", &sources).unwrap()
}

/// (table, line) for every relation that makes any statement in `sql` unbounded.
fn unbounded(sql: &str) -> Vec<(String, usize)> {
    let catalog = catalog();
    extract_sql_statement_facts(sql)
        .bounds
        .iter()
        .flat_map(|fact| offenders(fact, &catalog))
        .map(|offender| (offender.table, offender.line))
        .collect()
}

fn names(sql: &str) -> Vec<String> {
    unbounded(sql).into_iter().map(|(table, _)| table).collect()
}

#[test]
fn a_whole_unique_key_pinned_to_a_value_bounds_a_relation() {
    assert!(names("SELECT 1 FROM accounts WHERE id = $1").is_empty());
    assert!(names("SELECT 1 FROM accounts WHERE email = lower($1)").is_empty());
    assert!(names("SELECT 1 FROM public.accounts WHERE id = ANY($1)").is_empty());
    assert_eq!(
        names("SELECT 1 FROM accounts WHERE name = $1"),
        ["accounts"]
    );
    assert_eq!(names("SELECT 1 FROM accounts"), ["accounts"]);
    assert_eq!(
        names("SELECT 1 FROM order_lines WHERE line_no = $1"),
        ["order_lines"]
    );
}

#[test]
fn a_limit_a_pure_aggregate_or_an_unknown_relation_needs_no_key() {
    assert!(names("SELECT 1 FROM orders LIMIT 10").is_empty());
    assert!(names("SELECT count(*) FROM orders").is_empty());
    assert!(names("SELECT 1 FROM mystery_table").is_empty());
    assert!(names("SELECT 1 FROM unnest($1::int[]) AS x").is_empty());
    assert!(names("SELECT 1").is_empty());
    assert_eq!(
        names("SELECT status, count(*) FROM orders GROUP BY status"),
        ["orders"]
    );
}

#[test]
fn bounded_items_bound_the_relations_pinned_to_them() {
    // A chain: o is pinned by value, a by o, and nothing else is needed.
    assert!(
        names("SELECT 1 FROM orders o, accounts a WHERE o.id = $1 AND a.id = o.account_id")
            .is_empty()
    );
    // Equality between two unbounded relations bounds neither.
    assert_eq!(
        names("SELECT 1 FROM orders o, accounts a WHERE a.id = o.id AND o.id = a.id"),
        ["orders", "accounts"]
    );
    // A relation joined to a capped CTE by key is bounded by the CTE's limit.
    assert!(names(
        "WITH c AS (SELECT id FROM orders ORDER BY id LIMIT $1) \
         SELECT 1 FROM c JOIN accounts a ON a.id = c.id"
    )
    .is_empty());
    // An uncapped CTE reports its own tables and bounds nothing joined to it.
    assert_eq!(
        names("WITH c AS (SELECT id FROM orders) SELECT 1 FROM c JOIN accounts a ON a.id = c.id"),
        ["orders", "accounts"]
    );
}

#[test]
fn set_operations_need_every_arm_bounded() {
    assert!(names(
        "SELECT id FROM accounts WHERE id = $1 UNION ALL SELECT id FROM orders WHERE id = $1"
    )
    .is_empty());
    assert_eq!(
        names("SELECT id FROM accounts WHERE id = $1 UNION ALL SELECT id FROM orders"),
        ["orders"]
    );
    assert!(names("SELECT id FROM accounts UNION ALL SELECT id FROM orders LIMIT 5").is_empty());
}

#[test]
fn a_derived_table_is_bounded_when_its_query_is() {
    assert!(names("SELECT * FROM (SELECT id FROM orders ORDER BY id LIMIT 3) d").is_empty());
    assert_eq!(names("SELECT * FROM (SELECT id FROM orders) d"), ["orders"]);
}

#[test]
fn updates_and_deletes_need_only_their_target_bounded() {
    assert!(names("UPDATE orders SET status = 'x' WHERE id = $1").is_empty());
    assert_eq!(
        names("UPDATE orders SET status = 'x' WHERE account_id = $1"),
        ["orders"]
    );
    // The USING relation may be unbounded: each target row changes once.
    assert!(
        names("DELETE FROM orders o USING accounts a WHERE o.id = $1 AND a.name = o.status")
            .is_empty()
    );
    assert_eq!(
        names("DELETE FROM orders WHERE id IN (SELECT id FROM orders)"),
        ["orders"]
    );
    assert!(names("DELETE FROM orders WHERE id IN (SELECT id FROM orders LIMIT 4)").is_empty());
    // `ctid` identifies one row of any table.
    assert!(names("DELETE FROM mystery WHERE ctid = $1").is_empty());
    assert!(names("DELETE FROM sessions WHERE ctid = ANY($1)").is_empty());
    assert!(names("DELETE FROM sessions WHERE expires_at < now() LIMIT 5").is_empty());
}

#[test]
fn a_relation_the_catalog_names_in_another_form_is_found() {
    assert_eq!(names("SELECT 1 FROM public.orders"), ["orders"]);
    assert_eq!(
        names("SELECT 1 FROM \"orders\" AS o WHERE o.status = $1"),
        ["orders"]
    );
}

#[test]
fn offenders_carry_the_line_of_each_relation() {
    assert_eq!(
        unbounded("SELECT 1\n  FROM orders o\n  JOIN accounts a ON a.name = o.status"),
        [("orders".to_string(), 2), ("accounts".to_string(), 3)]
    );
}

#[test]
fn quoted_key_columns_match_quoted_sql_columns() {
    let catalog = SchemaCatalog::from_json(
        r#"{
          "formatVersion": 2, "coverage": "complete",
          "tables": { "mixed": {
            "columns": { "Order Id": { "dataType": "uuid" } },
            "indexes": { "mixed_pkey": {
              "unique": true, "primary": true, "valid": true, "ready": true,
              "keys": [ { "column": "Order Id", "expression": "\"Order Id\"" } ]
            } }
          } }
        }"#,
    )
    .unwrap();
    assert_eq!(catalog.unique_keys("mixed"), [vec!["Order Id".to_string()]]);
    let judge = |sql: &str| {
        extract_sql_statement_facts(sql)
            .bounds
            .iter()
            .flat_map(|fact| offenders(fact, &catalog))
            .count()
    };
    assert_eq!(judge("SELECT 1 FROM mixed WHERE \"Order Id\" = $1"), 0);
    assert_eq!(judge("SELECT 1 FROM mixed WHERE order_id = $1"), 1);
}

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-bounded-statements/fixture")
}

fn config(yaml: &str) -> NoMistakesConfig {
    NoMistakesConfig {
        rules: vec![RuleDef {
            rule: RULE_ID.to_string(),
            scope: Some(RuleScope::Repository),
            options: serde_yaml::from_str(yaml).unwrap(),
            ..Default::default()
        }],
        ..Default::default()
    }
}

#[test]
fn the_standalone_entry_points_prepare_their_own_facts() {
    let root = fixture_root();
    let files = [root.join("sql/invalid.sql"), root.join("schema.json")];
    let yaml =
        "schemaCatalogPath: schema.json\nsqlInclude: ['sql/invalid.sql']\nexclude: ['src/**']";
    let direct = check_with_files(&root, &config(yaml), &files).unwrap();
    assert_eq!(direct.len(), 3, "{direct:#?}");
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let shared = check_with_files_and_sources(&root, &config(yaml), &files, &sources).unwrap();
    assert_eq!(direct, shared);
}

#[test]
fn sql_that_cannot_be_analyzed_fails_closed_unless_ignored() {
    let root = fixture_root();
    let files = [
        root.join("sql/broken.sql"),
        root.join("src/dynamic.ts"),
        root.join("schema.json"),
    ];
    let base = "schemaCatalogPath: schema.json\nsqlInclude: ['sql/broken.sql']\n";
    let found: Vec<_> = check_with_files(&root, &config(base), &files)
        .unwrap()
        .into_iter()
        .map(|finding| (finding.file, finding.message, finding.target.unwrap()))
        .collect();
    assert_eq!(found.len(), 2, "{found:#?}");
    assert!(found.iter().all(|(_, _, target)| target == "unanalyzable"));
    assert!(found[0]
        .1
        .contains("SQL could not be analyzed for bounded statements"));
    assert!(found[1]
        .1
        .contains("not statically recoverable for bounded statements"));
    let ignored = format!("{base}unanalyzableSql: ignore\n");
    assert!(check_with_files(&root, &config(&ignored), &files)
        .unwrap()
        .is_empty());
}

#[test]
fn scanning_requires_prepared_facts() {
    let options: Options = serde_yaml::from_str("schemaCatalogPath: schema.json").unwrap();
    let compiled = compile_options(&options).unwrap();
    let sources = crate::codebase::rules::source_store_for_files(&[]);
    let error = super::scan::scan(Path::new("."), &compiled, &[], &sources, None)
        .err()
        .unwrap();
    assert!(error
        .to_string()
        .contains("prepared PostgreSQL facts are required"));
}
