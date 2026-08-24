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
