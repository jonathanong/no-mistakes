use super::super::{PostgresSqlAlterIndexOperation, PostgresSqlStatementKind};
use super::{facts, fixture};

#[test]
fn valid_metadata_and_adjacent_strings_keep_typed_identity_and_original_spans() {
    let sql = fixture("valid-metadata.sql");
    let result = facts("valid-metadata.sql");
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.statements.len(), 12);
    for statement in &result.statements {
        assert_eq!(
            statement.sql,
            sql[statement.span.start.offset..statement.span.end.offset]
        );
    }
    let PostgresSqlStatementKind::Comment { comment } = &result.statements[0].facts else {
        panic!()
    };
    assert_eq!(comment.name.sql, "example_function");
    assert_eq!(comment.arguments.as_ref().unwrap().len(), 0);
    assert_eq!(comment.comment.as_deref(), Some("documentation"));
    let PostgresSqlStatementKind::AlterIndex { index } = &result.statements[1].facts else {
        panic!()
    };
    assert_eq!(index.name.sql, "example_parent");
    assert!(
        matches!(&index.operation, PostgresSqlAlterIndexOperation::AttachPartition { partition } if partition.sql == "example_child")
    );
    let PostgresSqlStatementKind::Comment { comment } = &result.statements[2].facts else {
        panic!()
    };
    assert_eq!(comment.comment.as_deref(), Some("First. Second."));
    let PostgresSqlStatementKind::Comment { comment } = &result.statements[3].facts else {
        panic!()
    };
    assert_eq!(comment.name.parts[0].identity, "Schéma");
    let args = comment.arguments.as_ref().unwrap();
    assert_eq!(args[0].name.as_ref().unwrap().identity, "Name");
    assert_eq!(args[0].mode.as_deref(), Some("IN"));
    assert_eq!(args[1].mode.as_deref(), Some("VARIADIC"));
    assert_eq!(args[1].data_type.array_dimensions.len(), 1);
    assert_eq!(comment.comment.as_deref(), Some("雪's documentation"));
    let PostgresSqlStatementKind::Comment { comment } = &result.statements[4].facts else {
        panic!()
    };
    assert_eq!(comment.object_type, "PROCEDURE");
    assert!(comment.comment.is_none());
    let PostgresSqlStatementKind::Comment { comment } = &result.statements[5].facts else {
        panic!()
    };
    assert!(comment.arguments.is_none());
    let PostgresSqlStatementKind::AlterIndex { index } = &result.statements[6].facts else {
        panic!()
    };
    assert!(!index.if_exists);
    assert_eq!(index.name.parts[0].identity, "Schéma");
    let PostgresSqlStatementKind::AlterIndex { index } = &result.statements[7].facts else {
        panic!()
    };
    assert!(index.if_exists);
    assert!(
        matches!(&index.operation, PostgresSqlAlterIndexOperation::Rename { name } if name.parts[0].identity == "Renamed")
    );
    let PostgresSqlStatementKind::Comment { comment } = &result.statements[9].facts else {
        panic!()
    };
    assert_eq!(comment.comment.as_deref(), Some("abc"));
    assert!(matches!(
        result.statements[10].facts,
        PostgresSqlStatementKind::CreateTable { .. }
    ));
    let PostgresSqlStatementKind::Insert { insert } = &result.statements[11].facts else {
        panic!()
    };
    assert!(insert.complete);
}

#[test]
fn malformed_metadata_and_create_table_never_become_successful_facts() {
    let result = facts("invalid-metadata.sql");
    assert_eq!(result.diagnostics.len(), 23, "{:?}", result.diagnostics);
    assert_eq!(result.statements.len(), 1);
    assert_eq!(result.statements[0].ordinal, 23);
    assert!(matches!(
        result.statements[0].facts,
        PostgresSqlStatementKind::CreateIndex { .. }
    ));
}

#[test]
fn metadata_grammar_rejects_invalid_prefixes_even_without_dispatch_preconditions() {
    // The production dispatcher checks the prefix. These guards also protect
    // parser helpers from a future caller that supplies a different target.
    for sql in fixture("metadata-defensive.sql").lines() {
        let prepared = crate::codebase::postgres::parse::prepare_postgres_tokens(sql);
        let mut parser = sqlparser::parser::Parser::new(&sqlparser::dialect::PostgreSqlDialect {})
            .with_tokens_with_locations(prepared.tokens);
        let locations = super::super::locations::Locations::new(sql);
        assert!(super::super::metadata::collect(&mut parser, &locations).is_err());
    }
}

#[test]
fn comment_literal_families_and_nested_metadata_remain_typed() {
    let result = facts("comment-literals.sql");
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.statements.len(), 8);
    for (statement, expected) in result.statements.iter().zip([
        Some("dollar text"),
        Some("tagged text"),
        Some("doc"),
        Some("escaped\nline"),
        Some("table dollar"),
        Some("view"),
        None,
    ]) {
        let PostgresSqlStatementKind::Comment { comment } = &statement.facts else {
            panic!()
        };
        assert_eq!(comment.comment.as_deref(), expected);
    }
    let PostgresSqlStatementKind::DoBlock { block } = &result.statements[7].facts else {
        panic!()
    };
    let PostgresSqlStatementKind::Conditional { branches } = &block.statements[0].facts else {
        panic!()
    };
    assert!(matches!(
        branches[0].statements[0].facts,
        PostgresSqlStatementKind::Comment { .. }
    ));
}

#[test]
fn nested_unquoted_comment_value_is_diagnostic_and_preserves_neighbor() {
    let result = facts("nested-malformed-comment.sql");
    assert_eq!(result.statements.len(), 2);
    let PostgresSqlStatementKind::DoBlock { block } = &result.statements[0].facts else {
        panic!()
    };
    assert!(!block.complete);
    assert!(block.statements.is_empty());
    assert_eq!(block.diagnostics.len(), 1);
    assert!(block.diagnostics[0]
        .message
        .contains("COMMENT string or NULL"));
    assert!(matches!(
        result.statements[1].facts,
        PostgresSqlStatementKind::CreateIndex { .. }
    ));
    assert_eq!(result.statements[1].ordinal, 1);
}
