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
            options: crate::codebase::postgres::tests::fixture_rule_options(yaml),
            ..Default::default()
        }],
        ..Default::default()
    };
    let opts: Options = config.rules[0].try_rule_options()?;
    compile_options(&opts)
}

#[test]
fn defaults_require_only_the_catalog_path() {
    let opts = compiled("schemaCatalogPath: db/schema.json").unwrap();
    assert_eq!(opts.max_columns, 12);
    assert!(opts.check_returning);
    assert!(opts.relations.is_empty());
    assert!(opts.schema.sql_include.is_empty());
    assert_eq!(opts.whole_row_functions.len(), 5);
    assert!(opts.fail_unanalyzable);
}

#[test]
fn zero_max_columns_and_explicit_returning_are_kept() {
    let opts = compiled(
        "schemaCatalogPath: db/schema.json\nmaxColumns: 0\ncheckReturning: false\nallowWholeRowFunctions: []\n",
    )
    .unwrap();
    assert_eq!(opts.max_columns, 0);
    assert!(!opts.check_returning);
    assert!(opts.whole_row_functions.is_empty());
}

#[test]
fn config_errors_name_the_option() {
    let cases = [
        (
            "{}",
            "postgres-explicit-columns option schemaCatalogPath: required",
        ),
        (
            "schemaCatalogPath: db/schema.json\nmaxColumns: -1\n",
            "postgres-explicit-columns option maxColumns: must be zero or greater",
        ),
        (
            "schemaCatalogPath: db/schema.json\nrelations: ['']\n",
            "postgres-explicit-columns option relations: empty relation name",
        ),
        (
            "schemaCatalogPath: db/schema.json\nrelations: [accounts, Accounts]\n",
            "postgres-explicit-columns option relations: duplicate entry accounts",
        ),
        (
            "schemaCatalogPath: db/schema.json\nallowWholeRowFunctions: ['']\n",
            "postgres-explicit-columns option allowWholeRowFunctions: empty function name",
        ),
        (
            "schemaCatalogPath: db/schema.json\nallowWholeRowFunctions: [row_to_json, ROW_TO_JSON]\n",
            "postgres-explicit-columns option allowWholeRowFunctions: duplicate entry row_to_json",
        ),
        (
            "schemaCatalogPath: db/schema.json\nunanalyzableSql: sideways\n",
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
