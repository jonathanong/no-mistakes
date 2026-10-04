use super::names;

#[test]
fn a_whole_unique_key_pinned_to_a_value_bounds_a_relation() {
    assert!(names("SELECT 1 FROM accounts WHERE id = $1").is_empty());
    assert!(names("SELECT 1 FROM accounts WHERE email = lower($1)").is_empty());
    assert!(names("SELECT 1 FROM public.accounts WHERE id = ANY($1)").is_empty());
    assert_eq!(
        names("SELECT 1 FROM accounts WHERE name = $1"),
        ["accounts"]
    );
    assert_eq!(names("SELECT 1 FROM accounts"), ["accounts"]);
    assert_eq!(
        names("SELECT 1 FROM order_lines WHERE line_no = $1"),
        ["order_lines"]
    );
}

#[test]
fn a_limit_a_pure_aggregate_or_an_unknown_relation_needs_no_key() {
    assert!(names("SELECT 1 FROM orders LIMIT 10").is_empty());
    assert!(names("SELECT count(*) FROM orders").is_empty());
    assert!(names("SELECT 1 FROM mystery_table").is_empty());
    assert!(names("SELECT 1 FROM unnest($1::int[]) AS x").is_empty());
    assert!(names("SELECT 1").is_empty());
    assert_eq!(
        names("SELECT status, count(*) FROM orders GROUP BY status"),
        ["orders"]
    );
}
