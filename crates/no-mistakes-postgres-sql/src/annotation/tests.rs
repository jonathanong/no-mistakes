use super::sql_requires_query_annotation;

#[test]
fn missing_annotation_is_required() {
    assert!(sql_requires_query_annotation("SELECT id FROM posts"));
    assert!(sql_requires_query_annotation("  SELECT id FROM posts"));
}

#[test]
fn leading_block_comment_is_enough() {
    assert!(!sql_requires_query_annotation(
        "/* posts/list */ SELECT id FROM posts"
    ));
    assert!(!sql_requires_query_annotation("  /* name */\nSELECT 1"));
}

#[test]
fn empty_or_star_only_comments_are_not_annotations() {
    assert!(sql_requires_query_annotation("/* */ SELECT 1"));
    assert!(sql_requires_query_annotation("/**/ SELECT 1"));
}

#[test]
fn line_comments_are_not_annotations() {
    assert!(sql_requires_query_annotation("-- posts/list\nSELECT 1"));
}

#[test]
fn transaction_commands_are_exempt() {
    assert!(!sql_requires_query_annotation("BEGIN"));
    assert!(!sql_requires_query_annotation("  commit;"));
    assert!(!sql_requires_query_annotation("ROLLBACK TO SAVEPOINT s"));
    assert!(!sql_requires_query_annotation("/* tx */ BEGIN"));
}

#[test]
fn begin_in_a_select_still_requires_annotation() {
    assert!(sql_requires_query_annotation(
        "SELECT 'BEGIN' FROM posts WHERE body = 'begin'"
    ));
}
