use super::names;

#[test]
fn a_recovered_interpolation_pins_like_a_placeholder() {
    // A `${…}` interpolation reaches the rule as `sql_placeholder_N`.
    for sql in [
        "SELECT id FROM invoices WHERE id = sql_placeholder_1",
        "SELECT id FROM invoices WHERE sql_placeholder_1 = id",
        "SELECT id FROM invoices WHERE id = sql_placeholder_1::uuid",
        "SELECT 1 FROM order_lines WHERE order_id = sql_placeholder_1 AND line_no = sql_placeholder_2",
        "UPDATE exports SET s3_key = NULL WHERE id = sql_placeholder_1 AND expires_at IS NOT NULL",
        "SELECT id FROM orders WHERE id IN (sql_placeholder_1, sql_placeholder_2)",
        "SELECT id FROM orders WHERE id = ANY(sql_placeholder_1::uuid[])",
        "SELECT o.id FROM orders o JOIN accounts a ON a.id = o.account_id WHERE o.id = sql_placeholder_1",
    ] {
        assert!(names(sql).is_empty(), "{sql}");
    }
    // Only a key is pinned: a bind compared with another column is not.
    assert_eq!(
        names("SELECT 1 FROM invoices WHERE paid_at = sql_placeholder_1"),
        ["invoices"]
    );
}

#[test]
fn a_bound_propagates_across_joins_on_unique_keys() {
    // One hop, whichever side of the join and of the equality the key is on.
    for sql in [
        "SELECT 1 FROM accounts a JOIN profiles p ON p.account_id = a.id WHERE a.id = $1",
        "SELECT 1 FROM accounts a JOIN profiles p ON a.id = p.account_id WHERE a.id = $1",
        "SELECT 1 FROM profiles p JOIN accounts a ON a.id = p.account_id WHERE a.id = $1",
        "SELECT 1 FROM accounts a JOIN profiles p ON p.account_id = a.id WHERE a.id = sql_placeholder_1",
    ] {
        assert!(names(sql).is_empty(), "{sql}");
    }
    // Two hops, as explicit joins and as a comma list with the conditions in WHERE.
    for sql in [
        "SELECT 1 FROM orders o JOIN accounts a ON a.id = o.account_id JOIN profiles p ON p.account_id = a.id WHERE o.id = $1",
        "SELECT 1 FROM orders o, accounts a, profiles p WHERE o.id = $1 AND a.id = o.account_id AND p.account_id = a.id",
    ] {
        assert!(names(sql).is_empty(), "{sql}");
    }
    // A hop through a column that is not a whole key ends the chain.
    assert_eq!(
        names("SELECT 1 FROM accounts a JOIN orders o ON o.account_id = a.id WHERE a.id = $1"),
        ["orders"]
    );
    assert_eq!(
        names("SELECT 1 FROM accounts a JOIN orders o ON o.account_id = a.id JOIN order_lines l ON l.order_id = o.id WHERE a.id = $1"),
        ["orders", "order_lines"]
    );
}
