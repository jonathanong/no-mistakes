use super::*;
use crate::statement_facts::SqlLocationDemand;
use crate::statements::facts::{extract_sql_lock_scope_facts, StatementPolicySources};
use sqlparser::ast::Statement;

fn explicit_locks(sql: &str) -> Vec<RowAccess> {
    crate::parse::parse_postgres_sql(sql)
        .unwrap()
        .into_iter()
        .flat_map(|statement| match statement {
            Statement::Query(query) => collect_explicit(&query, None, None, false)
                .into_iter()
                .map(|(row, _)| row)
                .collect(),
            _ => Vec::new(),
        })
        .collect()
}

#[test]
fn relation_index_keeps_duplicate_names_ambiguous() {
    let locks = explicit_locks("SELECT 1 FROM app.items JOIN items ON true FOR UPDATE OF items");
    assert_eq!(locks.len(), 1);
    assert!(locks[0].relation.is_none());
}

#[test]
fn relation_index_resolves_qualified_and_quoted_names() {
    let qualified = explicit_locks("SELECT 1 FROM app.items FOR UPDATE OF app.items");
    assert_eq!(qualified[0].relation.as_deref(), Some("app.items"));
    let quoted =
        explicit_locks("SELECT 1 FROM notices AS \"NoticeAlias\" FOR UPDATE OF \"NoticeAlias\"");
    assert_eq!(quoted[0].relation.as_deref(), Some("notices"));
}

#[test]
fn parenthesized_conjuncts_project_keys_to_each_locked_alias() {
    let locks = explicit_locks(
        "SELECT 1 FROM notices n JOIN tasks t ON t.notice_id = n.id \
         WHERE (n.id = $1 AND (t.notice_id = $1)) FOR UPDATE OF n, t",
    );
    assert_eq!(locks.len(), 2);
    assert_eq!(locks[0].key, Some(("id".into(), "$1".into())));
    assert_eq!(locks[1].key, Some(("notice_id".into(), "$1".into())));
}

#[test]
fn saved_projection_sql_keeps_unsupported_shapes_unresolved_and_resolves_names() {
    let rows = explicit_locks(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-lock-written-rows-only/fixture/four-joined-aliases/lock_projection.sql"
    )));
    assert_eq!(rows.len(), 5);
    assert!(rows[..3].iter().all(|row| row.relation.is_none()));
    assert_eq!(rows[3].relation.as_deref(), Some("public.notices"));
    assert_eq!(rows[4].relation.as_deref(), Some("notices"));
}

fn share_lock_facts(sql: &str) -> crate::SqlStatementFileFacts {
    extract_sql_lock_scope_facts(
        sql,
        false,
        &[],
        SqlLocationDemand::None,
        StatementPolicySources {
            share_lock_writes: true,
            ..Default::default()
        },
    )
}

#[test]
fn repeated_explicit_locks_keep_share_regardless_of_clause_order() {
    let facts = share_lock_facts(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/postgres-facts/statements/explicit-lock-clause-order.sql"
    )));
    assert_eq!(facts.lock_writes.locks.len(), 2);
    assert_eq!(facts.lock_writes.share_statements.len(), 2);
    for (index, statement) in facts.lock_writes.share_statements.iter().enumerate() {
        let [lock] = statement.locks.as_slice() else {
            panic!("expected one effective SHARE lock per row");
        };
        assert_eq!(lock.mode, SequenceMode::Share);
        assert_eq!(lock.row, Some(index));
    }
}

#[test]
fn explicit_update_does_not_guarantee_a_lock_with_multiple_from_items() {
    let facts = share_lock_facts(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/postgres-facts/statements/explicit-update-from-items.sql"
    )));
    assert!(!facts.parse_failed);
    let [single, multiple] = facts.lock_writes.locks.as_slice() else {
        panic!("expected both UPDATE targets to resolve");
    };
    assert_eq!(single, multiple);
    assert_eq!(multiple.relation.as_deref(), Some("accounts"));
    assert_eq!(multiple.key, Some(("id".into(), "$1".into())));
    let [statement] = facts.lock_writes.share_statements.as_slice() else {
        panic!("only the single-table SELECT guarantees an UPDATE lock");
    };
    let [lock] = statement.locks.as_slice() else {
        panic!("expected one guaranteed UPDATE lock");
    };
    assert_eq!(lock.mode, SequenceMode::Update);
    assert_eq!(lock.row, Some(0));
}

#[test]
fn recovered_share_locks_keep_unknown_mode_when_batch_tokens_are_unavailable() {
    let facts = share_lock_facts(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/postgres-facts/statements/share-locks-lexical-recovery.sql"
    )));
    assert!(facts.parse_failed);
    assert_eq!(facts.lock_writes.locks.len(), 2);
    assert_eq!(facts.lock_writes.share_statements.len(), 2);
    for (index, statement) in facts.lock_writes.share_statements.iter().enumerate() {
        assert_eq!(
            facts.lock_writes.locks[index].relation.as_deref(),
            Some("accounts")
        );
        let [lock] = statement.locks.as_slice() else {
            panic!("expected one recovered SHARE lock per statement");
        };
        assert_eq!(lock.mode, SequenceMode::UnknownShare);
        assert_eq!(lock.row, Some(index));
    }
}

#[test]
fn a_literal_that_looks_like_a_placeholder_keeps_its_sql_quotes() {
    let facts = share_lock_facts(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/postgres-facts/statements/key-share-literal-colon.sql"
    )));
    assert!(!facts.parse_failed);
    let [lock] = facts.lock_writes.locks.as_slice() else {
        panic!("expected one row lock");
    };
    let [write] = facts.lock_writes.writes.as_slice() else {
        panic!("expected one row write");
    };
    assert_eq!(lock.key, Some(("id".into(), "':x'".into())));
    assert_eq!(write.key, lock.key);
}
