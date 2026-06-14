use super::ts_array::quoted_string_literal;

#[test]
fn quoted_string_literal_rejects_unclosed_literals() {
    assert!(quoted_string_literal("\"unterminated").is_none());
}
