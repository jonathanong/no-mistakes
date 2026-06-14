use super::extract::extract_ts_array_literal;
use super::ts_array::{quoted_string_literal, top_level_values};
use std::collections::BTreeSet;

#[test]
fn quoted_string_literal_rejects_unclosed_literals() {
    assert!(quoted_string_literal("\"unterminated").is_none());
}

#[test]
fn array_extraction_skips_trailing_comments_without_values() {
    assert_eq!(
        extract_ts_array_literal(
            r#"const NAMES = [
  "api",
  // trailing comment
];"#,
            "NAMES"
        ),
        BTreeSet::from(["api".to_string()])
    );
    assert_eq!(
        extract_ts_array_literal(
            r#"const NAMES = [
  "api",
  /* trailing block */
];"#,
            "NAMES"
        ),
        BTreeSet::from(["api".to_string()])
    );
    assert_eq!(
        top_level_values(
            r#"
"api",
/* unterminated
"#,
        ),
        vec!["\"api\"".to_string()]
    );
    assert!(top_level_values("// no newline").is_empty());
}
