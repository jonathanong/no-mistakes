use super::diff;

fn fixture(name: &str) -> Vec<u8> {
    std::fs::read(crate::test_support::fixture_root("changes-attributes").join(name)).unwrap()
}

#[test]
fn reports_sorted_escaped_paths_without_rounding_numbers() {
    assert_eq!(
        diff(&fixture("base.json"), &fixture("edit.json")),
        Some(
            [
                "/",
                "/portrait/crops",
                "/portrait/radius",
                "/portrait/trimmed",
                "/portrait/visited",
                "/portrait/~1label~0",
            ]
            .map(String::from)
            .to_vec()
        )
    );
}

#[test]
fn emits_parent_pointer_for_replaced_nested_subtrees() {
    let base = fixture("base.json");
    assert_eq!(
        diff(&base, &fixture("nested-replacement.json")),
        Some(vec!["/portrait".into()])
    );
    assert_eq!(
        diff(&base, &fixture("nested-empty-replacement.json")),
        Some(vec!["/portrait".into()])
    );
    assert_eq!(
        diff(
            &fixture("empty-nested-base.json"),
            &fixture("empty-nested-edit.json")
        ),
        Some(vec!["/keep".into()])
    );
}

#[test]
fn omits_attribute_paths_for_unreliable_diffs() {
    let base = fixture("base.json");
    for name in [
        "invalid.json",
        "replacement.json",
        "duplicate.json",
        "duplicate-in-array.json",
        "type-change.json",
        "reordered.json",
        "format-only.json",
        "array-root.json",
    ] {
        assert_eq!(diff(&base, &fixture(name)), None, "{name}");
    }
    assert_eq!(diff(&[0xff], &base), None);
    assert_eq!(diff(&base, &[0xff]), None);
    assert_eq!(diff(&fixture("invalid.json"), &base), None);
}
