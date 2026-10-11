use super::super::PostgresSqlStatementKind;
use super::{facts, fixture};

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
        let locations = Locations::new(&source.sql);
        let wrapper_context = super::super::wrappers::Context::new(
            &source,
            &locations,
            &prepared.recursive_views,
            &[],
            &[],
            super::super::wrappers::child::Markers {
                inserts: &[],
                index_only: Vec::new(),
            },
            super::super::metadata_preparation::Comments::new(),
        );
        let empty = fixture("empty.sql");
        assert!(conditional::project(
            &mut value,
            &tokens,
            &source,
            &Locations::new(&empty),
            &[],
            &prepared.recursive_views,
            &wrapper_context
        )
        .is_err());
        if name == "conditional-branch.sql" {
            assert!(conditional::project(
                &mut value,
                &[],
                &source,
                &Locations::new(&source.sql),
                &[],
                &prepared.recursive_views,
                &wrapper_context
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
                &prepared.recursive_views,
                &wrapper_context
            )
            .is_err());
        }
    }
}
