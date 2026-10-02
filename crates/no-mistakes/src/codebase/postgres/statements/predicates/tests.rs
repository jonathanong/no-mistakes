use crate::codebase::postgres::statements::extract_sql_statement_facts;

fn proven(sql: &str, table: &str, alias: Option<&str>) -> Vec<String> {
    relation(sql, table, alias).0
}

fn pending(sql: &str, table: &str, alias: Option<&str>) -> Vec<String> {
    relation(sql, table, alias).1
}

fn relation(sql: &str, table: &str, alias: Option<&str>) -> (Vec<String>, Vec<String>) {
    let facts = extract_sql_statement_facts(sql);
    let mut matches = Vec::new();
    for select in &facts.selects {
        matches.extend(select.relations.iter().filter(|relation| {
            same(
                relation.table.as_str(),
                table,
                relation.alias.as_deref(),
                alias,
            )
        }));
    }
    for group in facts.updates.iter().chain(facts.deletes.iter()) {
        matches.extend(group.iter().filter(|relation| {
            same(
                relation.table.as_str(),
                table,
                relation.alias.as_deref(),
                alias,
            )
        }));
    }
    let relation = matches
        .first()
        .unwrap_or_else(|| panic!("{sql} has no {table} {alias:?}: {facts:#?}"));
    (
        relation.constrained_columns.clone(),
        relation.unqualified_columns.clone(),
    )
}

fn same(table: &str, want_table: &str, alias: Option<&str>, want_alias: Option<&str>) -> bool {
    table.eq_ignore_ascii_case(want_table) && alias == want_alias
}

fn has(columns: &[String], column: &str) -> bool {
    columns.iter().any(|item| item.eq_ignore_ascii_case(column))
}

#[test]
fn invalid_examples_do_not_constrain_the_required_column() {
    assert!(!has(
        &proven("SELECT id FROM events WHERE kind = 'login'", "events", None),
        "account_id"
    ));
    assert!(!has(
        &proven(
            "SELECT e.id FROM events e JOIN accounts a ON a.id = e.owner_id",
            "events",
            Some("e")
        ),
        "account_id"
    ));
    assert!(!has(
        &proven(
            "SELECT id FROM events WHERE account_id = $1 OR kind = 'login'",
            "events",
            None
        ),
        "account_id"
    ));
    assert!(!has(
        &proven("DELETE FROM orders WHERE id = $1", "orders", None),
        "account_id"
    ));
    assert!(has(
        &proven("DELETE FROM orders WHERE id = $1", "orders", None),
        "id"
    ));
    assert!(!has(
        &proven(
            "SELECT o.id FROM orders o JOIN orders p ON p.account_id = $1 WHERE o.id = p.id",
            "orders",
            Some("o")
        ),
        "account_id"
    ));
    assert!(has(
        &proven(
            "SELECT o.id FROM orders o JOIN orders p ON p.account_id = $1 WHERE o.id = p.id",
            "orders",
            Some("p")
        ),
        "account_id"
    ));
}

#[test]
fn valid_examples_constrain_the_column() {
    assert!(has(
        &proven(
            "SELECT id FROM events WHERE account_id = $1",
            "events",
            None
        ),
        "account_id"
    ));
    assert!(has(
        &proven(
            "SELECT id FROM events WHERE account_id = ANY($1::uuid[]) AND kind = 'login'",
            "events",
            None
        ),
        "account_id"
    ));
    assert!(has(
        &proven(
            "SELECT e.id FROM accounts a JOIN events e ON e.account_id = a.id WHERE a.id = $1",
            "events",
            Some("e")
        ),
        "account_id"
    ));
    assert!(has(
        &proven(
            "SELECT id FROM events WHERE (account_id = $1 AND kind = 'a') OR (account_id = $2 AND kind = 'b')",
            "events",
            None
        ),
        "account_id"
    ));
    assert!(has(
        &proven(
            "UPDATE orders SET status = 'paid' WHERE account_id = $1 AND id = $2",
            "orders",
            None
        ),
        "account_id"
    ));
}

#[test]
fn comparisons_between_in_and_either_side_count() {
    assert!(has(
        &proven(
            "SELECT id FROM events WHERE account_id BETWEEN $1 AND $2",
            "events",
            None
        ),
        "account_id"
    ));
    assert!(has(
        &proven(
            "SELECT id FROM events WHERE $1 < account_id",
            "events",
            None
        ),
        "account_id"
    ));
    assert!(has(
        &proven(
            "SELECT id FROM events WHERE account_id IN (SELECT id FROM accounts)",
            "events",
            None
        ),
        "account_id"
    ));
    assert!(has(
        &proven(
            "SELECT o.id FROM orders o WHERE o.account_id >= $1",
            "orders",
            Some("o")
        ),
        "account_id"
    ));
}

#[test]
fn null_like_and_not_equal_do_not_count() {
    for sql in [
        "SELECT id FROM events WHERE account_id IS NULL",
        "SELECT id FROM events WHERE account_id IS NOT NULL",
        "SELECT id FROM events WHERE account_id <> $1",
        "SELECT id FROM events WHERE account_id LIKE $1",
        "SELECT id FROM events WHERE /* account_id = 1 */ kind = 'login'",
    ] {
        assert!(!has(&proven(sql, "events", None), "account_id"), "{sql}");
    }
}

#[test]
fn cte_is_not_the_table_and_schema_qualified_names_match() {
    let cte = extract_sql_statement_facts(
        "WITH events AS (SELECT 1 AS id) SELECT id FROM events WHERE account_id = $1",
    );
    assert!(cte.selects.iter().all(|select| select
        .relations
        .iter()
        .all(|relation| relation.table != "events")));
    assert!(has(
        &proven(
            "SELECT id FROM public.events WHERE account_id = $1",
            "public.events",
            None
        ),
        "account_id"
    ));
}

#[test]
fn insert_select_and_nested_selects_are_separate_facts() {
    let facts = extract_sql_statement_facts(
        "INSERT INTO archive (id) SELECT id FROM events WHERE kind = 'login'",
    );
    let select = facts
        .selects
        .iter()
        .find(|select| select.tables.iter().any(|table| table == "events"))
        .expect("events select");
    assert!(select.in_insert_select);
    assert!(!has(&select.relations[0].constrained_columns, "account_id"));
    let nested = extract_sql_statement_facts(
        "SELECT id FROM accounts WHERE id IN (SELECT id FROM events WHERE kind = 'login')",
    );
    assert!(nested.selects.iter().any(|select| {
        select.tables.iter().any(|table| table == "events")
            && select
                .relations
                .iter()
                .any(|relation| !has(&relation.constrained_columns, "account_id"))
    }));
}

#[test]
fn unqualified_columns_stay_pending_until_a_catalog_can_resolve_them() {
    let sql =
        "SELECT e.id FROM events e JOIN accounts a ON a.id = e.owner_id WHERE account_id = $1";
    assert!(!has(&proven(sql, "events", Some("e")), "account_id"));
    assert!(has(&pending(sql, "events", Some("e")), "account_id"));
}

#[test]
fn updates_and_deletes_are_grouped_per_statement() {
    let facts = extract_sql_statement_facts(
        "UPDATE orders SET status = 'paid' WHERE id = $1; DELETE FROM orders WHERE account_id = $1;",
    );
    assert_eq!(facts.updates.len(), 1);
    assert_eq!(facts.deletes.len(), 1);
    assert!(!has(&facts.updates[0][0].constrained_columns, "account_id"));
    assert!(has(&facts.deletes[0][0].constrained_columns, "account_id"));
}
