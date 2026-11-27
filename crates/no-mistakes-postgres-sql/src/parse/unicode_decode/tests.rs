use super::decode;

#[test]
fn decodes_doubled_quotes_escape_markers_and_long_scalars() {
    assert_eq!(
        decode("it''s !! !+01F600", '!').as_deref(),
        Some("it's ! 😀")
    );
}

#[test]
fn rejects_invalid_surrogates_and_incomplete_scalars() {
    assert!(decode("!D800!0041", '!').is_none());
    assert!(decode("!+110000", '!').is_none());
    assert!(decode("!12", '!').is_none());
}

#[test]
fn saved_surrogate_forms_accept_mixed_widths_and_reject_invalid_pairs() {
    let saved = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/postgres-facts/source/unicode-surrogate-escapes.txt"),
    )
    .unwrap();
    for line in saved.lines() {
        let (encoded, expected) = line.split_once('\t').unwrap();
        let expected = (expected != "INVALID").then_some(expected);
        assert_eq!(decode(encoded, '!').as_deref(), expected, "{encoded}");
    }
}
