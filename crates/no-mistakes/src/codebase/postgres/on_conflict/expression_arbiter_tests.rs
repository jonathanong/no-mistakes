use super::{judge_file, Catalog};
use crate::codebase::postgres::statement_facts::SqlStatementFileFacts;
use crate::codebase::postgres::statements::extract_sql_statement_facts;

fn findings(file: &SqlStatementFileFacts) -> Vec<(usize, String)> {
    judge_file(
        file,
        &Catalog {
            schema: &[],
            triggers: &[],
            replay_safe: &[],
            check_convergence: true,
            check_volatility: true,
            check_arbiter: true,
            check_triggers: true,
            check_generated: true,
            trigger_writes: &[],
        },
    )
}

fn messages(sql: &str) -> Vec<String> {
    findings(&extract_sql_statement_facts(sql))
        .into_iter()
        .map(|(_, message)| message)
        .collect()
}

#[test]
fn standalone_expression_arbiter_preserves_key_dependencies() {
    let unsafe_sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-idempotent-insert/fixture/expression-arbiters/sql/expression.sql"
    ));
    let safe_sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-idempotent-insert/fixture/expression-arbiters-safe/sql/excluded.sql"
    ));
    let parsed = extract_sql_statement_facts(unsafe_sql);
    assert!(!parsed.parse_failed);
    assert!(matches!(
        parsed.inserts[0].on_conflict.as_ref().unwrap().arbiter,
        crate::codebase::postgres::statement_facts::SqlConflictArbiter::Expressions { .. }
    ));
    assert!(messages(unsafe_sql)
        .iter()
        .any(|message| message.contains("expression arbiter depends on email")));
    assert!(messages(safe_sql).is_empty());
}

#[test]
fn coupled_expression_dependencies_must_move_together() {
    let partial = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-idempotent-insert/fixture/expression-arbiters/sql/coupled_partial.sql"
    ));
    let full = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-idempotent-insert/fixture/expression-arbiters-safe/sql/coupled_full.sql"
    ));
    let independent = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-idempotent-insert/fixture/expression-arbiters-safe/sql/independent_targets.sql"
    ));
    let parsed = extract_sql_statement_facts(partial);
    assert!(!parsed.parse_failed);
    assert!(matches!(
        &parsed.inserts[0].on_conflict.as_ref().unwrap().arbiter,
        crate::codebase::postgres::statement_facts::SqlConflictArbiter::Expressions {
            groups,
            unknown: false,
            ..
        } if groups == &[vec!["a".to_string(), "b".to_string()]]
    ));
    assert!(messages(partial)
        .iter()
        .any(|message| message.contains("every column in its expression")));
    assert!(messages(full).is_empty());
    assert!(messages(independent).is_empty());
}

#[test]
fn unsupported_function_arguments_do_not_claim_complete_dependencies() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/postgres-facts/statements/expression-arbiter-unsupported-functions.sql"
    ));
    let facts = extract_sql_statement_facts(sql);
    assert!(!facts.parse_failed);
    assert_eq!(facts.inserts.len(), 3);
    let found = findings(&facts);
    assert_eq!(found.len(), 3, "{found:?}");
    assert!(
        found
            .iter()
            .all(|(_, message)| message.contains("cannot prove")),
        "{found:?}"
    );
}

#[test]
fn expression_dependency_proof_shapes_are_conservative() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/postgres-facts/statements/expression-arbiter-proof-shapes.sql"
    ));
    let facts = extract_sql_statement_facts(sql);
    assert!(!facts.parse_failed);
    assert_eq!(facts.inserts.len(), 4);
    let found = findings(&facts);
    assert_eq!(found.len(), 4, "{found:?}");
    assert!(
        found[..3]
            .iter()
            .all(|(_, message)| message.contains("expression arbiter depends")),
        "{found:?}"
    );
    assert!(found[3].1.contains("cannot prove"), "{found:?}");
}

#[test]
fn wrapped_expression_arbiter_keeps_policy_fact() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/postgres-facts/statements/expression-arbiter-wrapped.sql"
    ));
    let facts = extract_sql_statement_facts(sql);
    assert!(!facts.parse_failed);
    assert_eq!(facts.inserts.len(), 1);
    let found = findings(&facts);
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(
        found[0].1.contains("expression arbiter depends on email"),
        "{found:?}"
    );
}

#[test]
fn lenient_recovery_preserves_expression_arbiter_policy() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-idempotent-insert/fixture/expression-arbiter-recovered/sql/malformed-neighbor.sql"
    ));
    let facts = extract_sql_statement_facts(sql);
    assert!(facts.parse_failed);
    assert_eq!(facts.inserts.len(), 1);
    let found = findings(&facts);
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(
        found[0].1.contains("expression arbiter depends on email"),
        "{found:?}"
    );
}
