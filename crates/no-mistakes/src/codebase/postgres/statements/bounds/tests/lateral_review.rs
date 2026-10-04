use super::facts;

#[test]
fn derived_outputs_do_not_shadow_reads_inside_their_own_queries() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/lateral-shadowed-outer-read.sql"
    ));
    let all = facts(sql);
    assert_eq!(all.len(), 7);
    let pins = |index: usize| &all[index].query.items[all[index].target.unwrap()].pins;
    for index in [0, 1, 4, 6] {
        // Every outer row finds itself, regardless of the inner LIMIT.
        assert!(pins(index).is_empty(), "statement {index}");
    }
    assert_eq!(pins(2).len(), 1);
    assert_eq!(pins(2)[0].reads.len(), 1);
    assert_eq!(pins(2)[0].reads[0].column, "id");
    assert_eq!(pins(2)[0].reads[0].tables, ["currencies"]);
    for index in [3, 5] {
        assert_eq!(pins(index).len(), 1, "statement {index}");
        assert!(pins(index)[0].reads.is_empty(), "statement {index}");
    }
}

#[test]
fn growing_preceding_scopes_preserve_local_lateral_ownership() {
    let cases = [
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/lateral-scope-growth-16.sql")),
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/lateral-scope-growth-32.sql")),
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/lateral-scope-growth-64.sql")),
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/lateral-scope-growth-128.sql")),
    ];
    for sql in cases {
        let all = facts(sql);
        let delete = all
            .iter()
            .find(|fact| fact.kind == super::SqlBoundKind::Delete)
            .unwrap();
        let pins = &delete.query.items[delete.target.unwrap()].pins;
        assert_eq!(pins.len(), 1);
        // Each qualified read belongs to a preceding inner source, not the deleted row.
        assert!(pins[0].reads.is_empty());
        assert!(matches!(&pins[0].source, super::SqlPinSource::Query(query) if query.capped));
    }
}
