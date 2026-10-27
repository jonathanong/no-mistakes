mod blocking_query_wrappers;
mod caller_array_provenance;
mod cardinality_basics;
mod catalog_identity_edges;
mod catalog_keys;
mod catalog_resolution;
mod conditional_table_function_inputs;
mod copy_quoted_table;
mod current_database;
mod ddl_compat_lifecycle;
mod ddl_search_path;
mod explicit_self_alias;
mod fetch_expressions;
mod fixture_configs;
mod json_function_outputs;
mod legacy_single_column;
mod multi_table_arms;
mod nested_positive_order;
mod nested_zero_order;
mod nonrecursive_ctes;
mod nullable_fixed_limit;
mod partitions;
mod pin_query_reads;
mod prepared;
mod prepared_inputs;
mod propagation;
mod recovered_table;
mod recursive_forward;
mod referential;
mod rejecting_having_projection;
mod review_table;
mod scalar_call_array;
mod scalar_case_projection;
mod schema_correlation;
mod search_path;
mod set_operation_limits;
mod shadow_drop;
mod standalone_table;
mod suppression;
mod table_only;
mod table_source_association;
mod temporary_bare_ownership;
mod trigger_scope;
mod wrapped_suppression;

use super::check_with_files;
use super::RULE_ID;
use crate::codebase::postgres::{extract_sql_statement_facts, SchemaCatalog};
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::PathBuf;

fn catalog() -> SchemaCatalog {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-bounded-statements/fixture");
    let sources = crate::codebase::rules::source_store_for_files(std::slice::from_ref(
        &root.join("schema.json"),
    ));
    SchemaCatalog::load(&root, "schema.json", &sources).unwrap()
}

#[test]
fn unique_keys_resolve_through_the_same_identity_as_the_table() {
    let root = fixture_root();
    let sources = crate::codebase::rules::source_store_for_files(std::slice::from_ref(
        &root.join("schema-qualified.json"),
    ));
    let catalog = SchemaCatalog::load(&root, "schema-qualified.json", &sources).unwrap();
    assert_eq!(catalog.unique_keys("widgets"), [vec!["id".to_string()]]);
}

pub(super) fn fixture_root() -> PathBuf {
    crate::test_support::rule_fixture_root("postgres-bounded-statements")
}

fn config(yaml: &str) -> NoMistakesConfig {
    NoMistakesConfig {
        rules: vec![RuleDef {
            rule: RULE_ID.to_string(),
            scope: Some(RuleScope::Repository),
            options: crate::codebase::postgres::tests::fixture_rule_options(yaml),
            ..Default::default()
        }],
        ..Default::default()
    }
}
