use super::normalize_sql_key;

#[test]
fn lowercases_keywords_and_unquoted_idents_only() {
    assert_eq!(
        normalize_sql_key("Deleted_At IS NULL"),
        "deleted_at is null"
    );
    assert_eq!(normalize_sql_key("status = 'ACTIVE'"), "status = 'ACTIVE'");
    assert_eq!(
        normalize_sql_key("\"Status\" = 'active'"),
        "\"Status\" = 'active'"
    );
}

#[test]
fn keeps_escaped_quotes_and_unclosed_quotes() {
    assert_eq!(normalize_sql_key("note = 'it''s'"), "note = 'it''s'");
    assert_eq!(normalize_sql_key("'open"), "'open");
    assert_eq!(normalize_sql_key("a <> 1"), "a < > 1");
}
