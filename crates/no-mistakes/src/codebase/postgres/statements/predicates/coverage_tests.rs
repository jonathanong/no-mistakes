use super::base_table;
use crate::codebase::postgres::statements::extract_sql_statement_facts;
use sqlparser::ast::{Ident, ObjectName, ObjectNamePart};

fn has_table(sql: &str, table: &str) -> bool {
    let facts = extract_sql_statement_facts(sql);
    facts
        .selects
        .iter()
        .any(|select| select.tables.iter().any(|name| name == table))
        || facts
            .updates
            .iter()
            .chain(facts.deletes.iter())
            .any(|group| {
                group
                    .iter()
                    .any(|relation| relation.table.eq_ignore_ascii_case(table))
            })
}

#[test]
fn writes_explain_joins_and_grouped_queries_are_recorded() {
    assert!(has_table(
        "UPDATE events e SET kind = 'x' FROM accounts a WHERE e.id = a.id",
        "events"
    ));
    assert!(has_table(
        "UPDATE events e SET kind = 'x' FROM accounts a JOIN owners o ON o.id = a.id WHERE e.id = a.id",
        "owners"
    ));
    assert!(has_table(
        "DELETE FROM orders USING accounts a WHERE orders.id = a.id",
        "accounts"
    ));
    assert!(has_table(
        "EXPLAIN ANALYZE UPDATE orders SET status = 'paid' WHERE id = 1",
        "orders"
    ));
    assert!(has_table(
        "UPDATE events e SET kind = 'x' FROM (SELECT id FROM accounts) a WHERE e.id = a.id",
        "accounts"
    ));
    assert!(has_table(
        "UPDATE events e SET kind = 'x' FROM (accounts a JOIN owners o ON o.id = a.id) WHERE e.id = a.id",
        "owners"
    ));
    assert!(has_table(
        "SELECT count(*) FROM events GROUP BY account_id",
        "events"
    ));
    assert!(has_table(
        "SELECT count(*) FROM events GROUP BY ALL",
        "events"
    ));
    let grouped =
        crate::codebase::postgres::parse_postgres_sql("SELECT count(*) FROM events GROUP BY ALL")
            .expect("GROUP BY ALL");
    let sqlparser::ast::Statement::Query(query) = &grouped[0] else {
        panic!("query");
    };
    let sqlparser::ast::SetExpr::Select(select) = query.body.as_ref() else {
        panic!("select");
    };
    assert!(matches!(
        select.group_by,
        sqlparser::ast::GroupByExpr::All(_)
    ));
    let called = extract_sql_statement_facts("SELECT id FROM events(1) WHERE account_id = 1");
    assert!(!called.parse_failed, "{called:#?}");
    assert!(called.selects.iter().any(|select| {
        select.tables.iter().any(|table| table == "events")
            && select
                .relations
                .iter()
                .all(|relation| relation.table != "events")
    }));
    let listed = extract_sql_statement_facts("SELECT id FROM events WHERE account_id IN (1, 2)");
    assert!(listed.selects[0].relations[0]
        .constrained_columns
        .iter()
        .any(|column| column == "account_id"));
    let constant = extract_sql_statement_facts("SELECT id FROM events WHERE 1 IN (1, 2)");
    assert!(constant.selects[0].relations[0]
        .constrained_columns
        .is_empty());
}

#[test]
fn mixed_proof_or_and_subquery_from_keep_pending_columns() {
    let mixed = extract_sql_statement_facts(
        "SELECT id FROM events e, orders o WHERE e.account_id = 1 OR account_id = 2",
    );
    let events = mixed.selects[0]
        .relations
        .iter()
        .find(|relation| relation.alias.as_deref() == Some("e"))
        .expect("events");
    assert!(events
        .unqualified_columns
        .iter()
        .any(|column| column == "account_id"));
    assert!(!events
        .constrained_columns
        .iter()
        .any(|column| column == "account_id"));
    let between = extract_sql_statement_facts(
        "SELECT id FROM events e, orders o WHERE account_id BETWEEN 1 AND 2",
    );
    assert!(between.selects[0].relations.iter().any(|relation| {
        relation
            .unqualified_columns
            .iter()
            .any(|column| column == "account_id")
    }));
    let derived = extract_sql_statement_facts(
        "SELECT id FROM events, (SELECT 1 AS id) s WHERE account_id = 1",
    );
    assert!(derived.selects[0].relations.iter().all(|relation| {
        !relation
            .constrained_columns
            .iter()
            .any(|column| column == "account_id")
    }));
}

#[test]
fn an_empty_table_name_is_not_a_base_table() {
    let name = ObjectName(vec![ObjectNamePart::Identifier(Ident::new(""))]);
    assert!(base_table(&name, &[]).is_none());
}
