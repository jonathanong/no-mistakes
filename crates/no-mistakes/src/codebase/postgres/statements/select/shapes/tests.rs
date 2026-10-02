use crate::codebase::postgres::statements::extract_sql_statement_facts;

fn not_in(sql: &str) -> Vec<usize> {
    extract_sql_statement_facts(sql)
        .selects
        .iter()
        .flat_map(|select| select.not_in_subqueries.clone())
        .collect()
}

fn counts(sql: &str) -> Vec<usize> {
    extract_sql_statement_facts(sql)
        .selects
        .iter()
        .flat_map(|select| {
            select
                .count_existence_checks
                .iter()
                .map(|count| count.line)
                .collect::<Vec<_>>()
        })
        .collect()
}

#[test]
fn invalid_examples_record_the_banned_shape() {
    assert_eq!(
        not_in("SELECT id FROM accounts WHERE id NOT IN (SELECT account_id FROM bans)"),
        vec![1]
    );
    assert_eq!(
        counts(
            "SELECT id FROM accounts a WHERE (SELECT COUNT(*) FROM orders o WHERE o.account_id = a.id) > 0"
        ),
        vec![1]
    );
    assert_eq!(
        counts("SELECT (SELECT COUNT(*) FROM orders WHERE account_id = $1) = 0 AS is_new"),
        vec![1]
    );
    assert_eq!(
        counts("SELECT COUNT(*) > 0 AS has_orders FROM orders WHERE account_id = $1"),
        vec![1]
    );
}

#[test]
fn valid_examples_are_not_recorded() {
    for sql in [
        "SELECT id FROM accounts a WHERE NOT EXISTS (SELECT 1 FROM bans b WHERE b.account_id = a.id)",
        "SELECT id FROM accounts WHERE id NOT IN (1, 2, 3)",
        "SELECT account_id FROM orders GROUP BY account_id HAVING COUNT(*) > 0",
        "SELECT (SELECT COUNT(*) FROM orders WHERE account_id = $1) > 5 AS is_frequent",
        "SELECT COUNT(*) AS n FROM orders",
    ] {
        assert!(not_in(sql).is_empty(), "{sql}");
        assert!(counts(sql).is_empty(), "{sql}");
    }
}

#[test]
fn not_around_in_subquery_and_distinct_count_are_recorded() {
    assert_eq!(
        not_in("SELECT id FROM accounts WHERE NOT (id IN (SELECT account_id FROM bans))"),
        vec![1]
    );
    assert!(not_in("SELECT id FROM accounts WHERE id IN (SELECT account_id FROM bans)").is_empty());
    assert_eq!(
        counts("SELECT COUNT(DISTINCT account_id) > 0 AS has_orders FROM orders"),
        vec![1]
    );
    assert_eq!(
        counts("SELECT id FROM accounts WHERE 0 < (SELECT COUNT(*) FROM orders)"),
        vec![1]
    );
    assert_eq!(
        counts("SELECT id FROM accounts WHERE (SELECT COUNT(*) FROM orders) <> 0"),
        vec![1]
    );
    assert_eq!(
        counts("SELECT id FROM accounts WHERE (SELECT COUNT(*) FROM orders) != 0"),
        vec![1]
    );
    assert_eq!(
        counts("SELECT id FROM accounts WHERE (SELECT COUNT(id) FROM orders) >= 1"),
        vec![1]
    );
    assert_eq!(
        counts("SELECT id FROM accounts WHERE (SELECT COUNT(*) FROM orders) < 1"),
        vec![1]
    );
    assert_eq!(
        counts("SELECT id FROM accounts WHERE (SELECT COUNT(*) FROM orders) <= 0"),
        vec![1]
    );
    assert_eq!(
        counts("SELECT CASE WHEN (SELECT COUNT(*) FROM orders) > 0 THEN 1 END"),
        vec![1]
    );
    assert_eq!(
        not_in(
            "SELECT a.id FROM accounts a JOIN bans b ON a.id NOT IN (SELECT account_id FROM bans)"
        ),
        vec![1]
    );
}

#[test]
fn count_inside_a_cte_is_recorded_on_the_cte_line() {
    let sql = "WITH recent AS (\n  SELECT (SELECT COUNT(*) FROM orders) > 0 AS has_orders\n)\nSELECT id FROM recent";
    let facts = extract_sql_statement_facts(sql);
    let lines: Vec<_> = facts
        .selects
        .iter()
        .flat_map(|select| {
            select
                .count_existence_checks
                .iter()
                .map(|count| count.line)
                .collect::<Vec<_>>()
        })
        .collect();
    // Line 2 is the CTE body. The outer `SELECT id FROM recent` is not a count.
    assert_eq!(lines, vec![2], "{facts:#?}");
}

#[test]
fn grouped_bare_count_is_not_an_existence_check() {
    assert!(counts("SELECT COUNT(*) > 0 FROM orders GROUP BY account_id").is_empty());
    assert!(counts("SELECT (SELECT COUNT(*) FROM orders GROUP BY account_id) > 0").is_empty());
}

#[test]
fn other_count_forms_are_not_existence_checks() {
    for sql in [
        "SELECT (SELECT COUNT(*) FROM orders) = 1",
        "SELECT (SELECT COUNT(*) FROM orders UNION SELECT COUNT(*) FROM bans) > 0",
        "SELECT (SELECT COUNT(*), 1 FROM orders) > 0",
        "SELECT (SELECT * FROM orders) > 0",
        "SELECT (SELECT 1 FROM orders) > 0",
        "SELECT (SELECT COUNT(*) FROM orders) > id FROM accounts",
        "SELECT (SELECT COUNT(*) FROM orders) = '0'",
    ] {
        assert!(counts(sql).is_empty(), "{sql}");
    }
}
