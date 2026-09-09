use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::{Path, PathBuf};

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/postgres/conflict-ordering")
}

fn fixture(scenario: &str) -> PathBuf {
    fixture_root().join("cases").join(scenario)
}

fn config() -> NoMistakesConfig {
    config_with_options("schemaCatalogPath: schema.json")
}

fn config_with_options(options: &str) -> NoMistakesConfig {
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: RULE_ID.to_string(),
        scope: Some(RuleScope::Repository),
        options: serde_yaml::from_str(options).unwrap(),
        ..Default::default()
    });
    config
}

fn files(root: &Path) -> Vec<PathBuf> {
    vec![root.join("src/insert.ts"), root.join("schema.json")]
}

fn findings(scenario: &str) -> Vec<RuleFinding> {
    let root = fixture(scenario);
    check_with_files(&root, &config(), &files(&root)).unwrap()
}

#[test]
fn rejects_a_multi_row_source_without_canonical_order() {
    let findings = findings("fail-missing-order");
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(
        findings[0].target.as_deref(),
        Some("missing-canonical-order")
    );
    assert!(findings[0].message.contains("catalog arbiter"));
}

#[test]
fn rejects_a_noncanonical_source_order() {
    let findings = findings("fail-noncanonical-order");
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].target.as_deref(), Some("noncanonical-order"));
}

#[test]
fn rejects_a_source_that_cannot_be_mapped_to_target_columns() {
    let findings = findings("fail-unresolved-source-order");
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(
        findings[0].target.as_deref(),
        Some("unresolved-source-order")
    );
}

#[test]
fn accepts_the_catalog_key_prefix() {
    assert!(findings("pass-canonical-order").is_empty());
}

#[test]
fn resolves_top_level_select_aliases_in_the_source_order() {
    let root = fixture("pass-canonical-order");
    let findings = check_with_files(
        &root,
        &config(),
        &[root.join("src/alias.ts"), root.join("schema.json")],
    )
    .unwrap();
    assert!(findings.is_empty(), "{findings:#?}");
}

#[test]
fn follows_the_shared_typed_transaction_executor_facts() {
    let root = fixture("pass-canonical-order");
    let findings = check_with_files(
        &root,
        &config(),
        &[root.join("src/transaction.ts"), root.join("schema.json")],
    )
    .unwrap();
    assert!(findings.is_empty(), "{findings:#?}");
}

#[test]
fn rejects_targetless_multi_row_do_nothing() {
    let findings = findings("fail-targetless");
    assert_eq!(findings[0].target.as_deref(), Some("targetless-arbiter"));
}

#[test]
fn accepts_expression_index_order_with_a_terminal_tie_breaker() {
    let findings = findings("pass-expression");
    assert!(findings.is_empty(), "{findings:#?}");
}

#[test]
fn rejects_differently_ordered_inferred_indexes() {
    let findings = findings("fail-ambiguous");
    assert_eq!(findings[0].target.as_deref(), Some("ambiguous-arbiter"));
}

#[test]
fn rejects_a_conflict_target_whose_written_order_differs_from_the_catalog() {
    let findings = findings("fail-reversed-target");
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].target.as_deref(), Some("noncanonical-target"));
}

#[test]
fn resolves_a_partial_unique_index_by_its_inference_predicate() {
    assert!(findings("pass-partial").is_empty());
    let findings = findings("fail-partial-predicate");
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].target.as_deref(), Some("unresolved-arbiter"));
}

#[test]
fn resolves_a_named_unique_constraint_to_its_catalog_key_order() {
    let findings = findings("pass-constraint");
    assert!(findings.is_empty(), "{findings:#?}");
}

#[test]
fn scans_opted_in_static_sql_sources() {
    let root = fixture("pass-sql-include");
    let findings = check_with_files(
        &root,
        &config_with_options(
            "schemaCatalogPath: schema.json\ninclude: ['src/**/*.ts']\nsqlInclude: ['queries/**/*.sql']",
        ),
        &[root.join("queries/insert.sql"), root.join("schema.json")],
    )
    .unwrap();
    assert!(findings.is_empty(), "{findings:#?}");
}

#[test]
fn scopes_sql_file_safe_directives_to_their_own_statement() {
    let root = fixture("fail-sql-statement-directive");
    let findings = check_with_files(
        &root,
        &config_with_options(
            "schemaCatalogPath: schema.json\ninclude: ['src/**/*.ts']\nsqlInclude: ['queries/**/*.sql']",
        ),
        &[root.join("queries/inserts.sql"), root.join("schema.json")],
    )
    .unwrap();
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(
        findings[0].target.as_deref(),
        Some("missing-canonical-order")
    );
    assert_eq!(findings[0].line, 5);
}

#[test]
fn requires_a_catalog_path() {
    let error = match compile_options(&Options::default()) {
        Ok(_) => panic!("missing schemaCatalogPath should fail"),
        Err(error) => error,
    };
    assert!(error.to_string().contains("schemaCatalogPath"));
}

#[test]
fn compile_options_canonicalize_executor_names_for_prepared_fact_lookup() {
    let compiled = compile_options(&Options {
        schema_catalog_path: "schema.json".to_string(),
        executor_names: vec![
            "write".to_string(),
            "query".to_string(),
            "write".to_string(),
        ],
        ..Default::default()
    })
    .unwrap();
    assert_eq!(compiled.embedded.executor_names, ["query", "write"]);
}

#[test]
fn prepared_scan_contextualizes_a_missing_embedded_sql_projection() {
    let root = fixture("pass-canonical-order");
    let file = root.join("src/insert.ts");
    let inventory = std::sync::Arc::new(crate::codebase::ts_source::FileInventory::from_paths(
        std::slice::from_ref(&file),
    ));
    let sources = crate::codebase::ts_source::SourceStore::new(inventory);
    let mut facts = crate::codebase::check_facts::CheckFactMap::default();
    facts.postgres_schema_catalogs.insert(
        "schema.json".to_string(),
        Ok(std::sync::Arc::new(
            crate::codebase::postgres::SchemaCatalog::default(),
        )),
    );
    let compiled = compile_options(&Options {
        schema_catalog_path: "schema.json".to_string(),
        ..Options::default()
    })
    .unwrap();

    let error = scan::scan_with_sources(
        &root,
        &compiled,
        std::slice::from_ref(&file),
        &sources,
        &facts,
    )
    .expect_err("missing prepared facts must fail closed");
    assert!(error.to_string().contains(
        "postgres-conflict-ordering failed to collect embedded SQL facts from prepared analysis"
    ));
    assert!(format!("{error:#}").contains(&file.display().to_string()));
}

#[test]
fn rejects_opaque_executor_arguments_by_default() {
    let findings = findings("fail-opaque-executor");
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].target.as_deref(), Some("unanalyzable-sql"));
}

#[test]
fn ignores_opaque_executor_arguments_when_unanalyzable_sql_is_ignore() {
    let root = fixture("fail-opaque-executor");
    let result = check_with_files(
        &root,
        &config_with_options("schemaCatalogPath: schema.json\nunanalyzableSql: ignore"),
        &files(&root),
    )
    .unwrap();
    assert!(result.is_empty(), "{result:#?}");
}

#[test]
fn rejects_recovered_dynamic_conflict_sql_by_default() {
    let root = fixture("pass-canonical-order");
    let result = check_with_files(
        &root,
        &config(),
        &[root.join("src/dynamic.ts"), root.join("schema.json")],
    )
    .unwrap();
    assert_eq!(result.len(), 1, "{result:#?}");
    assert_eq!(result[0].target.as_deref(), Some("unanalyzable-sql"));
}

#[test]
fn rejects_a_dynamic_insert_before_its_conflict_clause_is_recovered() {
    let root = fixture("pass-canonical-order");
    let result = check_with_files(
        &root,
        &config(),
        &[
            root.join("src/dynamic-insert-fragment.ts"),
            root.join("schema.json"),
        ],
    )
    .unwrap();
    assert_eq!(result.len(), 1, "{result:#?}");
    assert_eq!(result[0].target.as_deref(), Some("unanalyzable-sql"));
}

#[test]
fn ignores_recovered_dynamic_non_insert_sql() {
    let root = fixture("pass-canonical-order");
    let result = check_with_files(
        &root,
        &config(),
        &[root.join("src/dynamic-select.ts"), root.join("schema.json")],
    )
    .unwrap();
    assert!(result.is_empty(), "{result:#?}");
}

#[test]
fn ignores_dynamic_non_insert_sql_that_only_mentions_insert_in_comments_or_strings() {
    let root = fixture("pass-canonical-order");
    let result = check_with_files(
        &root,
        &config(),
        &[
            root.join("src/dynamic-comment-and-string.ts"),
            root.join("schema.json"),
        ],
    )
    .unwrap();
    assert!(result.is_empty(), "{result:#?}");
}

#[test]
fn allows_an_explicit_unanalyzable_sql_exception() {
    let root = fixture("pass-canonical-order");
    let result = check_with_files(
        &root,
        &config_with_options("schemaCatalogPath: schema.json\nunanalyzableSql: ignore"),
        &[root.join("src/dynamic.ts"), root.join("schema.json")],
    )
    .unwrap();
    assert!(result.is_empty(), "{result:#?}");
}

#[test]
fn honors_the_safe_directive_for_recovered_dynamic_sql() {
    let root = fixture("pass-canonical-order");
    let result = check_with_files(
        &root,
        &config(),
        &[root.join("src/dynamic-safe.ts"), root.join("schema.json")],
    )
    .unwrap();
    assert!(result.is_empty(), "{result:#?}");
}

#[test]
fn napi_check_reports_the_same_registered_rule() {
    let root = fixture("fail-missing-order");
    let report = crate::napi_api::check_json_impl(crate::napi_api::options::test_json_arg(
        serde_json::json!({ "root": root, "config": root.join(".no-mistakes.yml") }).to_string(),
    ))
    .unwrap();
    let report: serde_json::Value = serde_json::from_str(&report).unwrap();
    assert!(
        report["rules"].as_array().is_some_and(|findings| {
            findings.iter().any(|finding| {
                finding["rule"] == RULE_ID && finding["target"] == "missing-canonical-order"
            })
        }),
        "{report:#?}"
    );
}
