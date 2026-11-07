use super::*;
use sqlparser::ast::Statement;

fn explicit_locks(sql: &str) -> Vec<RowAccess> {
    let mut statements = crate::codebase::postgres::parse::parse_postgres_sql(sql).unwrap();
    let Statement::Query(query) = statements.remove(0) else {
        panic!("expected query");
    };
    collect_explicit(&query, None)
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
