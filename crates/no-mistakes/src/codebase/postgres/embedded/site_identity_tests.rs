use super::{placeholders, Invocations};

#[test]
fn invocation_ranges_survive_bind_renumbering_and_publication() {
    let source = std::fs::read_to_string(
        crate::test_support::rule_fixture_root("postgres-site-identity").join("identity.sql"),
    )
    .unwrap()
    .replace("sql_placeholder_", placeholders::INTERNAL_PLACEHOLDER);
    let boundary = source.find('\n').unwrap() + 1;
    let (prefix, suffix) = source.split_at(boundary);
    let mut first = Invocations::default();
    first.record(prefix.len(), 11);
    let mut second = Invocations::default();
    second.record(suffix.len(), 22);
    first.append(&second, suffix, 9, prefix.len());
    let sql = format!("{prefix}{}", placeholders::renumber_placeholders(suffix, 9));
    let sites = first.publish(&sql);
    let sql = placeholders::publish_placeholders(sql);
    assert_eq!(sites.at(&sql, (1, 1)), [11]);
    assert_eq!(sites.at(&sql, (2, 1)), [22]);
    assert_eq!(sites.at(&sql, (2, 26)), [22]);
    assert!(sites.at(&sql, (3, 1)).is_empty());
    assert!(sites.at(&sql, (99, 1)).is_empty());
}

#[test]
fn empty_helpers_do_not_claim_following_statement_sites() {
    let source = std::fs::read_to_string(
        crate::test_support::rule_fixture_root("postgres-site-identity").join("identity.sql"),
    )
    .unwrap();
    let mut sites = Invocations::default();
    sites.record(0, 11);
    assert!(sites.at(&source, (1, 1)).is_empty());
}

#[test]
fn cut_and_inserted_fragments_retain_the_enclosing_invocation() {
    let sql = std::fs::read_to_string(
        crate::test_support::rule_fixture_root("postgres-site-identity").join("identity.sql"),
    )
    .unwrap();
    let boundary = sql.find('\n').unwrap() + 1;
    let mut sites = Invocations::default();
    sites.record(sql.len(), 11);
    let sliced = sites.slice(boundary..sql.len());
    assert_eq!(sliced.at(&sql[boundary..], (1, 1)), [11]);
    assert!(sites.slice(sql.len()..sql.len()).is_empty());
    let mut inserted = Invocations::default();
    inserted.inherit(&sites, boundary, boundary);
    assert_eq!(inserted.at(&sql[..boundary], (1, 1)), [11]);
    inserted.inherit(&sites, sql.len(), boundary);
    assert_eq!(inserted.at(&sql[..boundary], (1, 1)), [11]);
}

#[test]
fn a_cloned_invocation_can_change_without_mutating_its_cached_origin() {
    let sql = std::fs::read_to_string(
        crate::test_support::rule_fixture_root("postgres-site-identity").join("identity.sql"),
    )
    .unwrap();
    let mut original = Invocations::default();
    original.record(sql.len(), 11);
    let mut branch = original.clone();
    branch.record(sql.len(), 22);
    assert_eq!(original.at(&sql, (1, 1)), [11]);
    assert_eq!(branch.at(&sql, (1, 1)), [11, 22]);
}
