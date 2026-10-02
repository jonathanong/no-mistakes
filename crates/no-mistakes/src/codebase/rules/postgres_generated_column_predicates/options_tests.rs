use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};

fn compiled(yaml: &str) -> Result<CompiledOptions> {
    let config = NoMistakesConfig {
        rules: vec![RuleDef {
            rule: RULE_ID.to_string(),
            scope: Some(RuleScope::Repository),
            options: serde_yaml::from_str(yaml).unwrap(),
            ..Default::default()
        }],
        ..Default::default()
    };
    let opts: Options = config.rules[0].try_rule_options()?;
    compile_options(&opts)
}

#[test]
fn defaults_track_uuid_extract_timestamp() {
    let opts = compiled("{}").unwrap();
    assert_eq!(opts.functions, ["uuid_extract_timestamp"]);
    assert!(opts.require_primary_key);
    assert!(opts.where_clause && opts.join_clause && opts.order_clause);
    assert!(opts.fail_unanalyzable);
    assert_eq!(opts.schema.sql_include, ["**/*.sql"]);
}

#[test]
fn a_named_function_list_is_kept() {
    let opts = compiled("functions: [uuid_extract_timestamp]\n").unwrap();
    assert_eq!(opts.functions, ["uuid_extract_timestamp"]);
}

#[test]
fn config_errors_name_the_option() {
    let cases = [
        (
            "functions: []\n",
            "postgres-generated-column-predicates option functions: must not be empty",
        ),
        (
            "functions: ['']\n",
            "postgres-generated-column-predicates option functions: empty function name",
        ),
        (
            "functions: [uuid_extract_timestamp, UUID_EXTRACT_TIMESTAMP]\n",
            "postgres-generated-column-predicates option functions: duplicate entry uuid_extract_timestamp",
        ),
        (
            "clauses: []\n",
            "postgres-generated-column-predicates option clauses: must not be empty",
        ),
        (
            "clauses: [sideways]\n",
            "postgres-generated-column-predicates option clauses: expected where, join, or order-by",
        ),
        (
            "clauses: [where, where]\n",
            "postgres-generated-column-predicates option clauses: duplicate entry where",
        ),
        (
            "clauses: ['']\n",
            "postgres-generated-column-predicates option clauses: empty clause name",
        ),
        (
            "extraGeneratedColumns:\n  - {table: orders, column: created_at, sourceColumn: id}\n  - {table: Orders, column: CREATED_AT, sourceColumn: id}\n",
            "postgres-generated-column-predicates option extraGeneratedColumns: duplicate entry orders.created_at",
        ),
        (
            "extraGeneratedColumns: [{table: '', column: created_at, sourceColumn: id}]\n",
            "postgres-generated-column-predicates option extraGeneratedColumns: empty table name",
        ),
        (
            "extraGeneratedColumns: [{table: orders, column: '', sourceColumn: id}]\n",
            "postgres-generated-column-predicates option extraGeneratedColumns: empty column name",
        ),
        (
            "extraGeneratedColumns: [{table: orders, column: created_at, sourceColumn: ''}]\n",
            "postgres-generated-column-predicates option extraGeneratedColumns: empty sourceColumn",
        ),
        (
            "unanalyzableSql: sideways\n",
            "unanalyzableSql must be `fail` or `ignore`",
        ),
    ];
    for (yaml, expected) in cases {
        let Err(error) = compiled(yaml) else {
            panic!("expected config error containing {expected}");
        };
        let error = error.to_string();
        assert!(error.contains(expected), "{error}");
    }
}
