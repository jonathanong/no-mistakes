use super::*;

#[test]
fn stdin_prose_and_unterminated_lexical_constructs_preserve_source() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-no-offset/fixture/review-followups/db");
    for name in [
        "lexical-boundaries.sql",
        "lexical-open-string.sql",
        "lexical-open-comment.sql",
        "lexical-open-dollar.sql",
        "lexical-malformed-dollar.sql",
    ] {
        let sql = std::fs::read_to_string(root.join(name)).unwrap();
        let normalized = normalize_copy_data(&sql);
        assert!(
            matches!(normalized, std::borrow::Cow::Borrowed(_)),
            "{name}"
        );
        assert_eq!(normalized, sql, "{name}");
    }
}
