use crate::codebase::postgres::extract_sql_statement_facts;

fn fixture(name: &str) -> String {
    let path = crate::test_support::rule_fixture_root("postgres-bounded-statements")
        .join("sql/cte-returning")
        .join(name);
    std::fs::read_to_string(path).unwrap()
}

fn bounds_len(name: &str) -> usize {
    let facts = extract_sql_statement_facts(&fixture(name));
    assert!(!facts.parse_failed, "{name}");
    facts.bounds.len()
}

#[test]
fn key_bounded_returning_and_select_ctes_bound_outer_primary_key_joins() {
    assert_eq!(bounds_len("select-pk.sql"), 1);
    for name in [
        "update-pk.sql",
        "delete-pk.sql",
        "nested-projection.sql",
        "nested-with-bounded.sql",
        "auxiliary-independent.sql",
        "no-returning-unused.sql",
    ] {
        assert_eq!(bounds_len(name), 2, "{name}");
    }
}

#[test]
fn unbounded_returning_and_nonkey_joins_do_not_gain_a_row_cap() {
    for name in [
        "unbounded-update.sql",
        "auxiliary-unbounded.sql",
        "nested-with-namesake.sql",
        "non-pk-join.sql",
        "recursive-opaque.sql",
    ] {
        assert_eq!(bounds_len(name), 2, "{name}");
    }
}

#[test]
fn modifying_ctes_cannot_borrow_a_later_siblings_bounded_projection() {
    assert_eq!(bounds_len("declaration-time-scope.sql"), 3);
}
