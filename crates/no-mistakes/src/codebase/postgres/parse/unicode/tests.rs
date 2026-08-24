use super::{decode_unicode_string, tokenize, tokenize_raw_unicode};

#[test]
fn preserves_raw_unicode_escapes_and_rejects_invalid_uescape_tokenization() {
    assert_eq!(decode_unicode_string("!0041", '!').as_deref(), Some("A"));
    assert!(tokenize("SELECT U&'\\D800'").is_empty());
    assert!(!tokenize_raw_unicode("SELECT U&identifier").is_empty());
    assert!(!tokenize_raw_unicode("SELECT U&'0041' UESCAPE marker").is_empty());
}
