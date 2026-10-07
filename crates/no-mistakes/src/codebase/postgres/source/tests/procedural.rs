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
fn conditional_occurrences_retain_branch_conditions_and_original_source() {
    let sql = fixture("conditional.sql");
    let result = facts("conditional.sql");
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    let PostgresSqlStatementKind::DoBlock { block } = &result.statements[0].facts else {
        panic!()
    };
    assert!(block.complete, "{:?}", block.diagnostics);
    assert_eq!(block.statements.len(), 2);
    let PostgresSqlStatementKind::Conditional { branches } = &block.statements[0].facts else {
        panic!()
    };
    assert_eq!(branches.len(), 3);
    assert!(branches[0].condition.is_some());
    assert!(branches[1].condition.is_some());
    assert!(branches[2].condition.is_none());
    for branch in branches {
        assert_eq!(branch.statements.len(), 1);
        let occurrence = &branch.statements[0];
        assert!(matches!(
            occurrence.facts,
            PostgresSqlStatementKind::AlterTable { .. }
        ));
        assert_eq!(
            &sql[occurrence.span.start.offset..occurrence.span.end.offset],
            occurrence.sql
        );
        assert!(occurrence.sql.starts_with("ALTER TABLE children"));
    }
}

#[test]
fn nested_conditional_generated_columns_retain_branch_local_storage() {
    let result = facts("conditional-nested.sql");
    assert!(result.diagnostics.is_empty());
    let PostgresSqlStatementKind::DoBlock { block } = &result.statements[0].facts else {
        panic!()
    };
    assert!(block.complete, "{:?}", block.diagnostics);
    let PostgresSqlStatementKind::Conditional { branches } = &block.statements[0].facts else {
        panic!()
    };
    let PostgresSqlStatementKind::Conditional { branches: nested } =
        &branches[0].statements[0].facts
    else {
        panic!()
    };
    for (statement, expected) in [
        (&nested[0].statements[0], "VIRTUAL"),
        (&nested[1].statements[0], "STORED"),
        (&branches[1].statements[0], "VIRTUAL"),
    ] {
        let PostgresSqlStatementKind::CreateTable { columns, .. } = &statement.facts else {
            panic!()
        };
        assert_eq!(
            columns[1].generated.as_ref().unwrap().storage.as_deref(),
            Some(expected)
        );
    }
}

#[test]
fn conditional_grammar_requires_a_procedural_source_owner() {
    let result = facts("conditional-outside-body.sql");
    assert_eq!(result.diagnostics.len(), 1);
    assert!(result.diagnostics[0].message.contains("procedural body"));
    assert_eq!(result.statements.len(), 1);
    assert!(matches!(
        result.statements[0].facts,
        PostgresSqlStatementKind::CreateTable { .. }
    ));
}

#[test]
fn procedural_begin_end_branches_keep_typed_occurrences() {
    let result = facts("conditional-block.sql");
    assert!(result.diagnostics.is_empty());
    let PostgresSqlStatementKind::DoBlock { block } = &result.statements[0].facts else {
        panic!()
    };
    assert!(block.complete, "{:?}", block.diagnostics);
    let PostgresSqlStatementKind::Conditional { branches } = &block.statements[0].facts else {
        panic!()
    };
    assert_eq!(branches[0].statements.len(), 1);
    assert!(branches[1].statements.is_empty());
}

#[test]
fn conditional_projection_requires_its_prepared_source_owner() {
    use super::super::{conditional, locations::Locations, PostgresSqlSource};
    use sqlparser::{ast::Statement, dialect::PostgreSqlDialect, parser::Parser, tokenizer::Token};
    for name in ["conditional-branch.sql", "conditional-empty.sql"] {
        let source = PostgresSqlSource {
            sql: fixture(name),
            file_name: None,
        };
        let prepared = crate::codebase::postgres::parse::prepare_postgres_tokens(&source.sql);
        let mut parser =
            Parser::new(&PostgreSqlDialect {}).with_tokens_with_locations(prepared.tokens);
        let Statement::If(mut value) = parser.parse_statement().unwrap() else {
            panic!()
        };
        let tokens = (0..parser.index())
            .map(|index| parser.token_at(index))
            .collect::<Vec<_>>();
        let empty = fixture("empty.sql");
        assert!(conditional::project(
            &mut value,
            &tokens,
            &source,
            &Locations::new(&empty),
            &[],
            &prepared.recursive_views
        )
        .is_err());
        if name == "conditional-branch.sql" {
            assert!(conditional::project(
                &mut value,
                &[],
                &source,
                &Locations::new(&source.sql),
                &[],
                &prepared.recursive_views
            )
            .is_err());
            let missing = tokens
                .iter()
                .copied()
                .filter(|token| token.token != Token::SemiColon)
                .collect::<Vec<_>>();
            assert!(conditional::project(
                &mut value,
                &missing,
                &source,
                &Locations::new(&source.sql),
                &[],
                &prepared.recursive_views
            )
            .is_err());
        }
    }
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
