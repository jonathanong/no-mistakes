use super::{decode_unicode_string, tokenize, tokenize_raw_unicode};

#[test]
fn preserves_raw_unicode_escapes_and_rejects_invalid_uescape_tokenization() {
    assert_eq!(decode_unicode_string("!0041", '!').as_deref(), Some("A"));
    assert!(tokenize("SELECT U&'\\D800'").is_empty());
    assert!(!tokenize_raw_unicode("SELECT U&identifier").is_empty());
    assert!(!tokenize_raw_unicode("SELECT U&'0041' UESCAPE marker").is_empty());
}

#[test]
fn tokenizes_uescape_markers_nested_comments_and_non_ascii_identifiers() {
    assert!(!tokenize("SELECT U&'!0041' UESCAPE '!'").is_empty());
    assert!(!tokenize("SELECT U&'0041' UESCAPE ''").is_empty());
    assert!(!tokenize("SELECT U&'0041' UESCAPE 'ab'").is_empty());
    assert!(!tokenize("SELECT U&'0041' UESCAPE").is_empty());
    assert!(!tokenize("SELECT U&'A\nB'").is_empty());
    assert!(!tokenize("SELECT /* /* nested */ x */ 1").is_empty());
    assert!(!tokenize("SELECT \"id\", _id, 你 FROM t").is_empty());
    assert!(!tokenize("SELECT U&'0041'").is_empty());
    assert!(tokenize("U & 'not-adjacent'")
        .iter()
        .all(|token| !matches!(token, sqlparser::tokenizer::Token::UnicodeStringLiteral(_))));
}
