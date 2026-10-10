use super::super::PostgresSqlStatementKind;
use super::{facts, fixture};

#[test]
fn procedural_body_occurrences_retain_origin_and_honest_control_flow_diagnostics() {
    let source = fixture("procedural.sql");
    let facts = facts("procedural.sql");
    assert_eq!(facts.statements.len(), 9);
    assert_eq!(facts.diagnostics.len(), 0);
    let blocks = facts
        .statements
        .iter()
        .filter_map(|statement| match &statement.facts {
            PostgresSqlStatementKind::DoBlock { block } => Some(block),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(blocks.len(), 8);
    assert!(blocks[0].complete);
    assert!(blocks[1].complete);
    let nested = &blocks[0].statements[0];
    assert_eq!(
        nested.sql,
        "ALTER TABLE \"Café\" ADD COLUMN \"雪\" integer;"
    );
    assert_eq!(
        &source[nested.span.start.offset..nested.span.end.offset],
        nested.sql
    );
    assert_eq!(nested.span.start.line, 2);
    assert!(blocks[2].complete);
    assert!(matches!(
        blocks[2].statements[0].facts,
        PostgresSqlStatementKind::Conditional { .. }
    ));
    for block in &blocks[3..5] {
        assert!(!block.complete);
        assert!(block.statements.is_empty());
        assert_eq!(block.diagnostics.len(), 1);
    }
    assert!(!blocks[5].complete);
    assert_eq!(blocks[5].statements.len(), 1);
    assert!(matches!(
        blocks[5].statements[0].facts,
        PostgresSqlStatementKind::CreateTable { .. }
    ));
    assert!(!blocks[6].complete);
    assert!(blocks[6].statements.is_empty());
    assert!(blocks[7].complete);
    assert_eq!(
        blocks[7].body_encoding,
        super::super::PostgresSqlBodyEncoding::SingleQuoted
    );
}

#[test]
fn deeply_nested_procedural_programs_report_a_bounded_source_boundary() {
    let facts = facts("procedural-depth.sql");
    assert!(facts.diagnostics.is_empty());
    let mut statement = &facts.statements[0];
    for _ in 0..64 {
        let PostgresSqlStatementKind::DoBlock { block } = &statement.facts else {
            panic!()
        };
        statement = &block.statements[0];
    }
    let PostgresSqlStatementKind::DoBlock { block } = &statement.facts else {
        panic!()
    };
    assert!(!block.complete);
    assert!(block.statements.is_empty());
    assert!(block.diagnostics[0].message.contains("safety limit"));
}

#[test]
fn malformed_procedural_envelopes_preserve_neighbors_and_report_body_boundaries() {
    let facts = facts("procedural-invalid.sql");
    assert_eq!(facts.diagnostics.len(), 5);
    assert_eq!(facts.statements.len(), 9);
    let blocks = facts
        .statements
        .iter()
        .filter_map(|statement| match &statement.facts {
            PostgresSqlStatementKind::DoBlock { block } => Some(block),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(blocks.len(), 4);
    for block in blocks {
        assert!(!block.complete);
        assert!(block.statements.is_empty());
        assert_eq!(block.diagnostics.len(), 1);
        assert!(block.diagnostics[0].span.is_some());
    }
}

#[test]
fn procedural_projection_rejects_tokens_borrowed_from_a_different_source_owner() {
    use super::super::{locations::Locations, procedural, PostgresSqlSource};
    use sqlparser::{dialect::PostgreSqlDialect, parser::Parser};
    let sql = fixture("unsupported.sql");
    let prepared = crate::codebase::postgres::parse::prepare_postgres_tokens(&sql);
    let mut parser = Parser::new(&PostgreSqlDialect {}).with_tokens_with_locations(prepared.tokens);
    let foreign = PostgresSqlSource {
        sql: fixture("procedural-foreign-source.sql"),
        file_name: None,
    };
    assert!(
        procedural::collect(&mut parser, &foreign, &Locations::new(&foreign.sql), 0)
            .unwrap_err()
            .contains("original source")
    );
    let prepared = crate::codebase::postgres::parse::prepare_postgres_tokens(&sql);
    let mut parser = Parser::new(&PostgreSqlDialect {}).with_tokens_with_locations(prepared.tokens);
    let empty = fixture("empty.sql");
    assert!(
        procedural::collect(&mut parser, &foreign, &Locations::new(&empty), 0)
            .unwrap_err()
            .contains("source span")
    );
}

#[test]
fn conditional_keyword_compatibility_preserves_sql_identifiers_and_case_expressions() {
    let result = facts("conditional-identifiers.sql");
    assert!(result.diagnostics.is_empty());
    let PostgresSqlStatementKind::DoBlock { block } = &result.statements[0].facts else {
        panic!()
    };
    assert!(block.complete, "{:?}", block.diagnostics);
    let PostgresSqlStatementKind::Conditional { branches } = &block.statements[0].facts else {
        panic!()
    };
    let PostgresSqlStatementKind::AlterTable { operations, .. } = &branches[0].statements[0].facts
    else {
        panic!()
    };
    let super::super::PostgresSqlAlterOperation::AddColumn { column, .. } = &operations[0] else {
        panic!()
    };
    assert_eq!(column.name.value, "elsif");
    assert_eq!(column.name.identity, "elsif");
    let PostgresSqlStatementKind::AlterTable { operations, .. } = &branches[0].statements[1].facts
    else {
        panic!()
    };
    let super::super::PostgresSqlAlterOperation::AddColumn { column, .. } = &operations[0] else {
        panic!()
    };
    assert!(column
        .generated
        .as_ref()
        .unwrap()
        .expression
        .sql
        .contains("THEN elsif"));
    let PostgresSqlStatementKind::AlterTable { operations, .. } = &branches[1].statements[0].facts
    else {
        panic!()
    };
    let super::super::PostgresSqlAlterOperation::AddColumn { column, .. } = &operations[0] else {
        panic!()
    };
    assert_eq!(column.name.value, "ELSIF");
    assert!(column.name.quoted);
}

#[test]
fn single_quoted_conditional_occurrences_keep_original_encoded_spans() {
    let sql = fixture("conditional-single-quoted.sql");
    let result = facts("conditional-single-quoted.sql");
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    let PostgresSqlStatementKind::DoBlock { block } = &result.statements[0].facts else {
        panic!()
    };
    assert!(block.complete, "{:?}", block.diagnostics);
    assert_eq!(
        block.body_encoding,
        super::super::PostgresSqlBodyEncoding::SingleQuoted
    );
    let PostgresSqlStatementKind::Conditional { branches } = &block.statements[0].facts else {
        panic!()
    };
    assert_eq!(branches.len(), 3);
    for branch in branches {
        let statement = &branch.statements[0];
        assert_eq!(
            &sql[statement.span.start.offset..statement.span.end.offset],
            statement.sql
        );
        assert!(statement.sql.starts_with("ALTER TABLE children"));
    }
    let statement = &block.statements[1];
    assert_eq!(
        &sql[statement.span.start.offset..statement.span.end.offset],
        statement.sql
    );
    assert!(statement.sql.contains("''snow''''s''"));
    let PostgresSqlStatementKind::AlterTable { operations, .. } = &statement.facts else {
        panic!()
    };
    let super::super::PostgresSqlAlterOperation::AddColumn { column } = &operations[0] else {
        panic!()
    };
    assert_eq!(column.name.value, "雪");
    let default = column.default.as_ref().unwrap();
    assert_eq!(default.sql, "'snow''s'");
    let span = default.span.as_ref().unwrap();
    assert_eq!(&sql[span.start.offset..span.end.offset], "''snow''''s''");
}

#[test]
fn literal_execute_owns_decoded_children_and_preserves_wrapper_neighbors() {
    use super::super::{PostgresSqlExecuteEncoding, PostgresSqlStatementKind::*};
    let sql = fixture("literal-execute.sql");
    let result = facts("literal-execute.sql");
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.statements.len(), 3);
    let DoBlock { block } = &result.statements[1].facts else {
        panic!()
    };
    assert!(!block.complete);
    assert_eq!(block.statements.len(), 6);
    for (index, count, encoding) in [
        (0, 2, PostgresSqlExecuteEncoding::DollarQuoted),
        (1, 1, PostgresSqlExecuteEncoding::SingleQuoted),
    ] {
        let wrapper = &block.statements[index];
        assert_eq!(
            &sql[wrapper.span.start.offset..wrapper.span.end.offset],
            wrapper.sql
        );
        let LiteralExecute { execute } = &wrapper.facts else {
            panic!()
        };
        assert!(execute.complete, "{:?}", execute.diagnostics);
        assert_eq!(execute.body_encoding, encoding);
        assert_eq!(execute.statements.len(), count);
        assert!(
            sql[execute.literal_span.start.offset..execute.literal_span.end.offset]
                .contains("INSERT INTO prompts")
        );
        for (ordinal, child) in execute.statements.iter().enumerate() {
            assert_eq!(child.ordinal, ordinal);
            assert!(matches!(child.facts, Insert { .. }));
            assert_eq!(
                &execute.decoded_sql[child.span.start.offset..child.span.end.offset],
                child.sql
            );
        }
    }
    let LiteralExecute { execute } = &block.statements[2].facts else {
        panic!()
    };
    assert!(!execute.complete);
    assert!(!execute.diagnostics.is_empty());
    assert!(execute.statements.is_empty());
    assert!(execute.diagnostics[0].span.as_ref().unwrap().end.offset <= execute.decoded_sql.len());
    for dynamic in &block.statements[3..] {
        assert!(matches!(dynamic.facts, Other));
    }
    let quoted = facts("literal-execute-outer-quoted.sql");
    let DoBlock { block } = &quoted.statements[0].facts else {
        panic!()
    };
    assert!(block.complete, "{:?}", block.diagnostics);
    let LiteralExecute { execute } = &block.statements[0].facts else {
        panic!()
    };
    assert_eq!(
        execute.decoded_sql,
        "INSERT INTO prompts (body) VALUES ('hi')"
    );
    assert!(matches!(execute.statements[0].facts, Insert { .. }));
}

#[test]
fn escaped_execute_strings_decode_postgres_escapes() {
    let result = facts("literal-execute-escaped.sql");
    let PostgresSqlStatementKind::DoBlock { block } = &result.statements[0].facts else {
        panic!()
    };
    assert!(block.complete, "{:?}", block.diagnostics);
    let PostgresSqlStatementKind::LiteralExecute { execute } = &block.statements[0].facts else {
        panic!()
    };
    assert_eq!(
        execute.body_encoding,
        super::super::PostgresSqlExecuteEncoding::EscapedString
    );
    assert!(matches!(
        execute.statements[0].facts,
        PostgresSqlStatementKind::Insert { .. }
    ));
}

#[test]
fn execute_projection_bounds_nesting_and_requires_source_coordinates() {
    use super::super::{execute, locations::Locations, PostgresSqlSource};
    use sqlparser::{dialect::PostgreSqlDialect, parser::Parser};
    let source = PostgresSqlSource {
        sql: fixture("execute-only.sql"),
        file_name: None,
    };
    let prepared = crate::codebase::postgres::parse::prepare_postgres_tokens(&source.sql);
    let mut parser =
        Parser::new(&PostgreSqlDialect {}).with_tokens_with_locations(prepared.tokens.clone());
    let PostgresSqlStatementKind::LiteralExecute { execute } =
        execute::collect(&mut parser, &source, &Locations::new(&source.sql), 64).unwrap()
    else {
        panic!()
    };
    assert!(!execute.complete);
    assert!(execute.diagnostics[0].message.contains("safety limit"));
    let mut parser = Parser::new(&PostgreSqlDialect {}).with_tokens_with_locations(prepared.tokens);
    assert!(
        execute::collect(&mut parser, &source, &Locations::new(""), 1)
            .unwrap_err()
            .contains("source span")
    );
}

#[test]
fn conditional_execute_occurrences_use_the_same_literal_projection() {
    use super::super::PostgresSqlStatementKind::*;
    let result = facts("literal-execute-conditional.sql");
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.statements.len(), 2);
    let DoBlock { block } = &result.statements[0].facts else {
        panic!()
    };
    assert!(matches!(block.statements[1].facts, Select { .. }));
    let Conditional { branches } = &block.statements[0].facts else {
        panic!()
    };
    let Conditional { branches: nested } = &branches[2].statements[0].facts else {
        panic!()
    };
    for statement in [
        &branches[0].statements[0],
        &branches[1].statements[0],
        &nested[0].statements[1],
    ] {
        let LiteralExecute { execute } = &statement.facts else {
            panic!("{:?}", statement.facts)
        };
        assert!(execute.complete);
        assert!(matches!(execute.statements[0].facts, Insert { .. }));
    }
    assert!(matches!(branches[0].statements[1].facts, Insert { .. }));
    assert!(matches!(nested[0].statements[0].facts, Other));
    assert!(!block.complete);
}
