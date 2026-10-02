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

#[test]
fn unicode_dollar_tag_keeps_copy_payload_inside_the_literal() {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/rules/postgres-no-offset/fixture/review-followups/db/unicode-dollar.sql",
    );
    let sql = std::fs::read_to_string(path).unwrap();
    let normalized = normalize_copy_data(&sql);
    assert!(matches!(normalized, std::borrow::Cow::Borrowed(_)));
    assert!(normalized.contains("PAYLOAD_ROW"));
    assert_eq!(
        crate::codebase::postgres::sql_file_offset_uses(&sql),
        vec![(5, crate::codebase::postgres::OffsetUse::Other)]
    );
}

#[test]
fn dollar_tags_use_unicode_identifier_characters() {
    let mut line = 1usize;
    let quoted = "$é$\n$é$";
    assert_eq!(
        super::dollar::skip(quoted.as_bytes(), 0, &mut line),
        quoted.len()
    );
    assert_eq!(line, 2);
    assert_eq!(super::dollar::skip(b"$1$", 0, &mut line), 1);
    assert_eq!(super::dollar::skip(b"$_a1_$x$_a1_$", 0, &mut line), 13);
    assert_eq!(
        super::dollar::skip(b"$tag$unterminated", 0, &mut line),
        b"$tag$unterminated".len()
    );
    assert_eq!(super::dollar::skip(b"\xFF$", 0, &mut line), 1);
    assert_eq!(super::dollar::skip(b"x", 0, &mut line), 1);
    assert_eq!(super::dollar::skip(b"$", 0, &mut line), 1);
    let mid = "é$tag$".as_bytes();
    assert_eq!(super::dollar::skip(mid, 1, &mut line), 2);
    assert_eq!(super::dollar::skip(mid, 2, &mut line), 3);
    assert_eq!(super::dollar::skip(b"a_$tag$x$tag$", 2, &mut line), 3);
    let dash = "—$tag$x$tag$";
    let dollar = dash.find('$').unwrap();
    assert_eq!(
        super::dollar::skip(dash.as_bytes(), dollar, &mut line),
        dash.len()
    );
}
