use super::super::PostgresSqlStatementKind;
use super::{facts, fixture};

#[test]
fn e_continuation_escaped_quotes_preserve_text_unicode_spans_and_neighbors() {
    let sql = fixture("escaped-quote-continuations.sql");
    let result = facts("escaped-quote-continuations.sql");
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.statements.len(), 11);
    for (statement, expected) in result.statements.iter().zip([
        "ab'c",
        "雪'quoted'",
        "first'segment\\path",
        "ab'c\\",
        "xline\nlast'quote",
        "E'a'\n 'plain'",
        "E'a'\n 'b\\'c'",
        "after comment \"double\" and 'single'",
    ]) {
        let PostgresSqlStatementKind::Comment { comment } = &statement.facts else {
            panic!()
        };
        assert_eq!(comment.comment.as_deref(), Some(expected));
    }
    for statement in &result.statements {
        assert_eq!(
            statement.sql,
            sql[statement.span.start.offset..statement.span.end.offset]
        );
    }
    assert!(matches!(
        result.statements[8].facts,
        PostgresSqlStatementKind::CreateTable { .. }
    ));
    assert!(matches!(
        result.statements[9].facts,
        PostgresSqlStatementKind::CreateIndex { .. }
    ));
}

#[test]
fn escaped_continuation_masking_does_not_accept_block_same_line_or_unclosed_controls() {
    let result = facts("escaped-quote-continuations-invalid.sql");
    assert_eq!(result.statements.len(), 2);
    assert!(result.statements.iter().all(|statement| matches!(
        statement.facts,
        PostgresSqlStatementKind::CreateIndex { .. }
    )));
    assert!(result.diagnostics.len() >= 3);
    assert!(result
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("Unterminated")));
}

#[test]
fn dollar_tag_underscores_and_eof_comments_preserve_metadata() {
    let result = facts("escaped-quote-continuations.sql");
    let PostgresSqlStatementKind::Comment { comment } = &result.statements[10].facts else {
        panic!()
    };
    assert_eq!(comment.comment.as_deref(), Some("tag with underscores"));
    assert!(result.diagnostics.is_empty());
}

#[test]
fn incomplete_initial_escape_string_remains_lexical_failure() {
    let result = facts("escaped-quote-initial-unterminated.sql");
    assert!(result.statements.is_empty());
    assert!(result
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("Unterminated")));
}
