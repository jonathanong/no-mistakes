use crate::codebase::postgres::statements::extract_sql_statement_facts;
use crate::codebase::postgres::SqlColumnClause;

fn uses(sql: &str) -> Vec<(String, String, SqlColumnClause)> {
    extract_sql_statement_facts(sql)
        .selects
        .into_iter()
        .flat_map(|select| select.column_uses)
        .map(|use_| (use_.table, use_.column, use_.clause))
        .collect()
}

#[test]
fn invalid_examples_record_where_and_order_by() {
    assert_eq!(
        uses("SELECT id FROM orders WHERE created_at > $1"),
        vec![("orders".into(), "created_at".into(), SqlColumnClause::Where)]
    );
    assert_eq!(
        uses("SELECT id FROM orders ORDER BY created_at DESC LIMIT 20"),
        vec![(
            "orders".into(),
            "created_at".into(),
            SqlColumnClause::OrderBy
        )]
    );
    assert_eq!(
        uses("SELECT o.id FROM orders o WHERE o.created_at BETWEEN $1 AND $2"),
        vec![("orders".into(), "created_at".into(), SqlColumnClause::Where)]
    );
}

#[test]
fn valid_examples_skip_display_null_functions_and_plain_sorts() {
    assert!(
        uses("SELECT id, created_at FROM orders ORDER BY id DESC LIMIT 20")
            .iter()
            .all(|use_| use_.1 == "id")
    );
    assert_eq!(
        uses("SELECT id FROM orders WHERE id > $1"),
        vec![("orders".into(), "id".into(), SqlColumnClause::Where)]
    );
    assert!(uses("SELECT id FROM orders WHERE created_at IS NULL").is_empty());
    assert!(
        uses("SELECT date_trunc('day', created_at) AS d, COUNT(*) FROM orders GROUP BY d")
            .is_empty()
    );
    assert_eq!(
        uses("SELECT id FROM invoices ORDER BY created_at"),
        vec![(
            "invoices".into(),
            "created_at".into(),
            SqlColumnClause::OrderBy
        )]
    );
}

#[test]
fn joins_record_on_and_leave_ambiguous_names_unassigned() {
    assert_eq!(
        uses("SELECT o.id FROM orders o JOIN tags t ON o.created_at = t.id"),
        vec![
            ("orders".into(), "created_at".into(), SqlColumnClause::Join),
            ("tags".into(), "id".into(), SqlColumnClause::Join),
        ]
    );
    assert_eq!(
        uses(
            "SELECT id FROM orders JOIN invoices ON orders.id = invoices.id WHERE created_at > $1"
        ),
        vec![
            (String::new(), "created_at".into(), SqlColumnClause::Where),
            ("orders".into(), "id".into(), SqlColumnClause::Join),
            ("invoices".into(), "id".into(), SqlColumnClause::Join),
        ]
    );
}
