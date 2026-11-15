use super::shape;

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
