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
fn omits_attribute_paths_for_unreliable_diffs() {
    let base = fixture("base.json");
    for name in [
        "invalid.json",
        "replacement.json",
        "duplicate.json",
        "type-change.json",
        "reordered.json",
        "format-only.json",
        "array-root.json",
    ] {
        assert_eq!(diff(&base, &fixture(name)), None, "{name}");
    }
}
