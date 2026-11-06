use super::shape;

#[test]
fn an_interpolation_recovered_from_a_template_is_a_bind() {
    // `${id}` reaches the facts as `sql_placeholder_N`: a value, never a column.
    assert_eq!(
        shape("SELECT id FROM images WHERE id = sql_placeholder_1"),
        ["select: images[id=value]"]
    );
    assert_eq!(
        shape("SELECT id FROM images WHERE sql_placeholder_1::uuid = id"),
        ["select: images[id=value]"]
    );
    assert_eq!(
        shape("SELECT id FROM images WHERE id = ANY(sql_placeholder_1)"),
        ["select: images[id=value]"]
    );
    assert_eq!(
        shape("SELECT 1 FROM a JOIN b ON b.a_id = a.id WHERE a.id = sql_placeholder_1 AND b.k IN (sql_placeholder_2, 3)"),
        ["select: a[id=#1 id=value] b[a_id=#0 k=value a_id=value]"]
    );
    // A bind never names a column of the item it is compared with.
    assert_eq!(
        shape(
            "SELECT 1 FROM a, b WHERE a.id = sql_placeholder_1 AND b.id = sql_placeholder_1 + a.n"
        ),
        ["select: a[id=value] b[id=#0]"]
    );
}
