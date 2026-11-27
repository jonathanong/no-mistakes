use super::shape;
use crate::statements::SqlPinSource;

#[test]
fn a_single_value_group_is_capped_and_the_negative_shapes_are_not() {
    assert_eq!(
        shape("SELECT id FROM entries WHERE id = $1 GROUP BY id"),
        ["select: capped entries[id=value]"]
    );
    let carried = shape(
        "WITH current_version AS (
            SELECT post.id AS post_id FROM entries post WHERE post.id = $1
        )
        SELECT post_id FROM current_version GROUP BY post_id",
    );
    assert!(
        carried.len() == 1 && carried[0].starts_with("select: capped "),
        "{carried:?}"
    );
    for sql in [
        "SELECT id FROM entries GROUP BY id",
        "SELECT id FROM entries WHERE id = $1 GROUP BY ROLLUP(id)",
        "SELECT id FROM entries WHERE id = $1 GROUP BY 1",
        "SELECT id FROM entries WHERE id IN ($1, $2) GROUP BY id",
        "SELECT id FROM entries WHERE id = $1 OR id = $2 GROUP BY id",
        "SELECT id, generate_series(1, 2) FROM entries WHERE id = $1 GROUP BY id",
        "SELECT id FROM entries WHERE id = $1 GROUP BY id ORDER BY generate_series(1, 3)",
    ] {
        let found = shape(sql);
        assert!(
            found.iter().all(|fact| !fact.contains("capped")),
            "{sql} => {found:?}"
        );
    }
}

#[test]
fn a_two_column_in_list_is_not_one_peer() {
    let facts = crate::extract_sql_statement_facts(
        "SELECT a.id FROM flows a, flows b, flows c WHERE a.id IN (b.id, c.id)",
    );
    let pins = &facts.bounds[0].query.items[0].pins;
    assert!(
        pins.iter().any(|pin| {
            matches!(&pin.source, SqlPinSource::Items(items) if items.len() != 1)
                && pin.reads.is_empty()
                && pin.qualified_reads.is_empty()
                && pin.expression.is_none()
                && pin.alternative.is_none()
                && !pin.null_safe
                && !pin.unqualified_candidate
        }),
        "{pins:?}"
    );
}
