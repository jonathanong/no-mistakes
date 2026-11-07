use super::*;

#[test]
fn complete_statement_prefixes_retain_the_enclosing_query_context() {
    for sql in [
        "SELECT 1",
        "SELECT id FROM items WHERE (expires_at < $1",
        "WITH selected AS (SELECT id FROM items WHERE expires_at < $1",
        "SELECT (SELECT id FROM items WHERE (expires_at < $1",
        "SELECT coalesce(1",
        "SELECT coalesce(1) FROM items WHERE (expires_at < $1",
    ] {
        assert!(complete_prefix(sql), "{sql}");
    }
}

#[test]
fn required_or_unrecoverable_prefixes_do_not_allow_masking() {
    for sql in [
        "",
        "SELECT",
        "SELECT id FROM",
        "SELECT coalesce(",
        "SELECT first,",
        "SELECT id FROM (",
        "SELECT 1 ) broken",
        "SELECT bad(, value",
        "SELECT 'unterminated",
    ] {
        assert!(!complete_prefix(sql), "{sql}");
    }
}
