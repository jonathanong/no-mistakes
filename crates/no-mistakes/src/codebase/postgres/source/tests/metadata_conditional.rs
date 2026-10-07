use super::super::PostgresSqlStatementKind;
use super::{facts, fixture};

#[test]
fn conditional_routine_comments_preserve_typed_signatures_and_source_positions() {
    let sql = fixture("conditional-routine-comment.sql");
    let result = facts("conditional-routine-comment.sql");
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    let PostgresSqlStatementKind::DoBlock { block } = &result.statements[0].facts else {
        panic!()
    };
    assert!(block.complete, "{:?}", block.diagnostics);
    assert_eq!(block.statements.len(), 2);
    let PostgresSqlStatementKind::Conditional { branches } = &block.statements[0].facts else {
        panic!()
    };
    let routine = &branches[0].statements[0];
    assert_eq!(
        routine.sql,
        sql[routine.span.start.offset..routine.span.end.offset]
    );
    let PostgresSqlStatementKind::Comment { comment } = &routine.facts else {
        panic!()
    };
    assert_eq!(comment.object_type, "FUNCTION");
    assert_eq!(comment.name.parts[0].identity, "Schéma");
    assert_eq!(comment.name.parts[1].identity, "Fun");
    assert_eq!(comment.comment.as_deref(), Some("雪"));
    let args = comment.arguments.as_ref().unwrap();
    assert_eq!(args.len(), 2);
    assert_eq!(args[0].name.as_ref().unwrap().identity, "Name");
    assert_eq!(args[0].mode.as_deref(), Some("IN"));
    assert_eq!(args[1].mode.as_deref(), Some("VARIADIC"));
    assert_eq!(args[1].data_type.array_dimensions.len(), 1);
    let PostgresSqlStatementKind::Comment { comment } = &branches[0].statements[1].facts else {
        panic!()
    };
    assert!(comment.arguments.is_none());
    let PostgresSqlStatementKind::Conditional { branches } = &branches[0].statements[2].facts
    else {
        panic!()
    };
    let PostgresSqlStatementKind::Comment { comment } = &branches[0].statements[0].facts else {
        panic!()
    };
    assert_eq!(comment.object_type, "PROCEDURE");
    assert!(comment.comment.is_none());
    assert_eq!(comment.arguments.as_ref().unwrap().len(), 1);
    let PostgresSqlStatementKind::Comment { comment } = &branches[1].statements[0].facts else {
        panic!()
    };
    assert_eq!(comment.arguments.as_ref().unwrap().len(), 0);
    let PostgresSqlStatementKind::Comment { comment } = &block.statements[1].facts else {
        panic!()
    };
    assert_eq!(comment.name.sql, "direct");
    assert_eq!(comment.arguments.as_ref().unwrap().len(), 1);
    assert!(matches!(
        result.statements[1].facts,
        PostgresSqlStatementKind::CreateIndex { .. }
    ));
}

#[test]
fn malformed_conditional_signatures_and_boundaries_fail_closed_and_keep_neighbors() {
    let result = facts("conditional-routine-comment-invalid.sql");
    assert_eq!(result.statements.len(), 4);
    for index in [0, 2] {
        let PostgresSqlStatementKind::DoBlock { block } = &result.statements[index].facts else {
            panic!()
        };
        assert!(!block.complete);
        assert!(!block.diagnostics.is_empty());
        assert!(block.statements.is_empty());
        assert!(matches!(
            result.statements[index + 1].facts,
            PostgresSqlStatementKind::CreateIndex { .. }
        ));
    }
}

#[test]
fn prepared_comment_collection_resets_the_borrowed_parser_cursor() {
    let sql = fixture("metadata-preparation-cursor.sql");
    let prepared = crate::codebase::postgres::parse::prepare_postgres_tokens(&sql);
    let mut parser = sqlparser::parser::Parser::new(&sqlparser::dialect::PostgreSqlDialect {})
        .with_tokens_with_locations(prepared.tokens);
    // A caller may have inspected its first token before handing over ownership.
    parser.next_token();
    let locations = super::super::locations::Locations::new(&sql);
    let (parser, comments) = super::super::metadata_preparation::prepare(parser, &locations);
    assert_eq!(parser.index(), 0);
    assert_eq!(comments.len(), 2);
    assert!(comments.values().all(|value| matches!(
        value.facts,
        Some(Ok(PostgresSqlStatementKind::Comment { .. }))
    )));
}
