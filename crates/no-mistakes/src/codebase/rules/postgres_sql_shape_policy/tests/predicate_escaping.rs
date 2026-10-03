use super::bounded_iteration::{at, fixture, found};

#[test]
fn escaped_delimiters_cannot_turn_a_value_into_a_non_selective_expression() {
    let options =
        std::fs::read_to_string(fixture("predicate-escaping").join("options.yml")).unwrap();
    assert_eq!(
        found("predicate-escaping", &options, "sql/001.sql"),
        at(&[
            (2, "keyset-only-sweep"),
            (5, "keyset-only-sweep"),
            (8, "keyset-only-sweep"),
            (11, "keyset-only-sweep")
        ])
    );
}

#[test]
fn equivalent_literal_spellings_match_without_colliding_with_other_values() {
    let options =
        std::fs::read_to_string(fixture("predicate-escaping").join("options-normalization.yml"))
            .unwrap();
    assert_eq!(
        found("predicate-escaping", &options, "sql/002.sql"),
        at(&[
            (2, "keyset-only-sweep"),
            (4, "keyset-only-sweep"),
            (6, "keyset-only-sweep"),
            (8, "keyset-only-sweep"),
            (10, "keyset-only-sweep"),
        ])
    );
}
