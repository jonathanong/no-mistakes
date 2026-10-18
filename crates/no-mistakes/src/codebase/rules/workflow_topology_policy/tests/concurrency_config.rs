use super::super::concurrency_compile::{self, CompiledIntent};
use anyhow::Result;
use std::collections::BTreeMap;

fn options(yaml: &str) -> super::super::Options {
    super::config(yaml).rules[0].try_rule_options().unwrap()
}

fn compile(yaml: &str) -> Result<BTreeMap<String, CompiledIntent>> {
    concurrency_compile::compile(&options(yaml).concurrency_policy)
}

fn error(yaml: &str) -> String {
    compile(yaml).unwrap_err().to_string()
}

#[test]
fn missing_pending_cancellation_and_scope_are_config_errors() {
    let pending = error(
        r#"
concurrencyPolicy:
  ci.yml:
    cancellation: retain-running
    scope: [run]
"#,
    );
    assert!(
        pending.contains("workflow-topology-policy option concurrencyPolicy: missing pending"),
        "{pending}"
    );
    let cancellation = error(
        r#"
concurrencyPolicy:
  ci.yml:
    pending: fifo
    scope: [run]
"#,
    );
    assert!(
        cancellation.contains("missing cancellation"),
        "{cancellation}"
    );
    let scope = error(
        r#"
concurrencyPolicy:
  ci.yml:
    pending: fifo
    cancellation: retain-running
"#,
    );
    assert!(scope.contains("missing scope"), "{scope}");
}

#[test]
fn unknown_pending_cancellation_and_scope_are_config_errors() {
    let pending = error(
        r#"
concurrencyPolicy:
  ci.yml:
    pending: soon
    cancellation: retain-running
    scope: [run]
"#,
    );
    assert!(pending.contains("unknown pending: soon"), "{pending}");
    let cancellation = error(
        r#"
concurrencyPolicy:
  ci.yml:
    pending: fifo
    cancellation: maybe
    scope: [run]
"#,
    );
    assert!(
        cancellation.contains("unknown cancellation: maybe"),
        "{cancellation}"
    );
    let scope = error(
        r#"
concurrencyPolicy:
  ci.yml:
    pending: fifo
    cancellation: retain-running
    scope: [branch]
"#,
    );
    assert!(scope.contains("unknown scope: branch"), "{scope}");
}

#[test]
fn duplicate_empty_and_combined_fixed_resource_scopes_are_config_errors() {
    let duplicate = error(
        r#"
concurrencyPolicy:
  ci.yml:
    pending: fifo
    cancellation: retain-running
    scope: [sha, sha]
"#,
    );
    assert!(duplicate.contains("duplicate scope: sha"), "{duplicate}");
    let empty = error(
        r#"
concurrencyPolicy:
  ci.yml:
    pending: fifo
    cancellation: retain-running
    scope: []
"#,
    );
    assert!(empty.contains("empty scope"), "{empty}");
    let combined = error(
        r#"
concurrencyPolicy:
  ci.yml:
    pending: fifo
    cancellation: retain-running
    scope: [fixed-resource, sha]
"#,
    );
    assert!(
        combined.contains("fixed-resource combined with another scope"),
        "{combined}"
    );
}

#[test]
fn valid_policy_normalizes_scope_order_and_fixed_resource() {
    let compiled = compile(
        r#"
concurrencyPolicy:
  ci.yml:
    pending: coalesce-latest
    cancellation: conditional
    scope: [sha, pull-request]
  deploy.yml:
    pending: fifo
    cancellation: retain-running
    scope: [fixed-resource]
"#,
    )
    .unwrap();
    assert_eq!(compiled["ci.yml"].pending, "coalesce-latest");
    assert_eq!(compiled["ci.yml"].cancellation, "conditional");
    assert_eq!(
        compiled["ci.yml"].scope,
        vec!["pull-request".to_string(), "sha".to_string()]
    );
    assert_eq!(compiled["deploy.yml"].pending, "fifo");
    assert!(compiled["deploy.yml"].scope.is_empty());
}
