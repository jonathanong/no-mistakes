fn texts(index: usize) -> Vec<String> {
    super::fixture::texts("edges.ts", index)
}

#[test]
fn same_argument_keeps_only_the_diagonal() {
    assert_eq!(
        texts(0),
        [
            "DELETE FROM users WHERE id = $1",
            "DELETE FROM posts WHERE id = $1"
        ]
    );
}

#[test]
fn two_closed_arguments_publish_their_product() {
    assert_eq!(
        texts(1),
        [
            "DELETE FROM a_w WHERE id = $1",
            "DELETE FROM a_x WHERE id = $1",
            "DELETE FROM b_w WHERE id = $1",
            "DELETE FROM b_x WHERE id = $1"
        ]
    );
}

#[test]
fn product_over_the_variant_budget_stays_opaque() {
    let values = texts(2);
    assert!(values.is_empty(), "{values:?}");
}

#[test]
fn const_object_argument_does_not_expand_its_fields() {
    assert_eq!(texts(3), ["DELETE FROM literal WHERE id = $1"]);
}
