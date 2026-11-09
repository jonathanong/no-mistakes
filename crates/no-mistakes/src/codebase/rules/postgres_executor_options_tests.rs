use crate::config::v2::schema::{NoMistakesConfig, RuleDef};

fn decode<T: for<'de> serde::Deserialize<'de> + Default>(rule: &RuleDef) {
    rule.try_rule_options::<T>().unwrap();
}

#[test]
fn every_executor_rule_decodes_extensions_with_its_actual_option_type() {
    let config: NoMistakesConfig = serde_yaml::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/config/postgres-executor-names/extra.yml"
    )))
    .unwrap();
    let mut rule = config.rules[0].clone();
    macro_rules! check {
        ($module:ident) => {{
            rule.rule = stringify!($module).replace('_', "-");
            decode::<super::$module::Options>(&rule);
        }};
    }
    check!(postgres_no_test_state_mutation);
    check!(postgres_bounded_statements);
    check!(postgres_explicit_columns);
    check!(postgres_generated_column_predicates);
    check!(postgres_no_generated_column_writes);
    check!(postgres_required_predicates);
    check!(postgres_sql_shape_policy);
    check!(postgres_no_offset);
    check!(postgres_no_transformed_uuid_predicates);
    check!(postgres_conflict_ordering);
    check!(postgres_lock_ordering);
    check!(postgres_idempotent_insert);
    check!(postgres_require_query_annotation);
    check!(postgres_sql_statement_policy);
    check!(postgres_json_relation_ids);
    check!(postgres_grouping_not_aggregation);
    rule.rule = "postgres-require-query-annotation".into();
    decode::<super::postgres_unmatched_executors::Options>(&rule);
    decode::<super::postgres_undeclared_executors::Options>(&rule);
    for id in [
        "postgres-no-mutable-sql-registry",
        "postgres-no-unbounded-sql-identifier",
    ] {
        rule.rule = id.into();
        decode::<super::postgres_sql_cross_file::Options>(&rule);
    }
}
