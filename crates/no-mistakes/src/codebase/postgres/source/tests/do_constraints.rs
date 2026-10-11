use super::super::{PostgresSqlAlterOperation, PostgresSqlStatement, PostgresSqlStatementKind};
use super::{facts, fixture};

fn occurrences(statement: &PostgresSqlStatement, output: &mut Vec<PostgresSqlStatement>) {
    output.push(statement.clone());
    let nested = match &statement.facts {
        PostgresSqlStatementKind::DoBlock { block } => block.statements.iter().collect(),
        PostgresSqlStatementKind::Conditional { branches } => branches
            .iter()
            .flat_map(|branch| &branch.statements)
            .collect(),
        _ => Vec::new(),
    };
    for statement in nested {
        occurrences(statement, output);
    }
}

#[test]
fn nested_do_constraints_keep_typed_names_validation_and_exact_source_provenance() {
    for name in [
        "nested-constraint-plain.sql",
        "nested-constraints.sql",
        "nested-constraints-single.sql",
    ] {
        let sql = fixture(name);
        let result = facts(name);
        assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
        let mut all = Vec::new();
        for statement in &result.statements {
            occurrences(statement, &mut all);
        }
        let constraints = all
            .iter()
            .filter_map(|statement| match &statement.facts {
                PostgresSqlStatementKind::AlterTable { table, operations } => {
                    Some((statement, table, operations))
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(
            constraints.len(),
            if name == "nested-constraint-plain.sql" {
                1
            } else {
                3
            }
        );
        let (_, table, operations) = constraints[0];
        let PostgresSqlAlterOperation::AddConstraint {
            constraint,
            not_valid,
        } = &operations[0]
        else {
            panic!()
        };
        assert!(*not_valid);
        assert_eq!(
            constraint.kind,
            super::super::PostgresSqlConstraintKind::ForeignKey
        );
        let plain = name == "nested-constraint-plain.sql";
        let expected_name = if plain { "sample_constraint" } else { "FK雪" };
        let constraint_name = constraint.name.as_ref().unwrap();
        assert_eq!(constraint_name.value, expected_name);
        assert_eq!(constraint_name.identity, expected_name);
        assert_eq!(constraint_name.quoted, !plain);
        assert_eq!(
            table
                .parts
                .iter()
                .map(|part| part.value.as_str())
                .collect::<Vec<_>>(),
            if plain {
                vec!["sample_child"]
            } else {
                vec!["Mý", "Chïld"]
            }
        );
        assert_eq!(
            constraint.columns[0].value,
            if plain { "parent_id" } else { "Parent Id" }
        );
        let constraint_span = constraint.span.as_ref().expect("constraint source range");
        assert!(
            sql[constraint_span.start.offset..constraint_span.end.offset].starts_with("CONSTRAINT")
        );
        assert!(!sql[constraint_span.start.offset..constraint_span.end.offset].ends_with(';'));
        let referenced = constraint.referenced_table.as_ref().unwrap();
        assert_eq!(
            referenced
                .parts
                .iter()
                .map(|part| part.value.as_str())
                .collect::<Vec<_>>(),
            if plain {
                vec!["sample_parent"]
            } else {
                vec!["Mý", "Parent"]
            }
        );
        assert_eq!(
            constraint.referenced_columns[0].value,
            if plain { "id" } else { "Id" }
        );
        let block_statement = result
            .statements
            .iter()
            .find(|statement| matches!(statement.facts, PostgresSqlStatementKind::DoBlock { .. }))
            .unwrap();
        let PostgresSqlStatementKind::DoBlock { block } = &block_statement.facts else {
            panic!()
        };
        assert!(block.complete);
        assert!(block.diagnostics.is_empty());
        for (statement, _, _) in &constraints {
            assert!(statement.span.start.offset > block_statement.span.start.offset);
            assert!(statement.span.end.offset < block_statement.span.end.offset);
            assert!(statement.span.start.offset >= block.body_span.start.offset);
            assert!(statement.span.end.offset <= block.body_span.end.offset);
        }
        if !plain {
            assert_eq!(
                result
                    .statements
                    .iter()
                    .map(|statement| statement.ordinal)
                    .collect::<Vec<_>>(),
                [0, 1, 2]
            );
            for (index, expected) in [
                (1, super::super::PostgresSqlConstraintKind::Unique),
                (2, super::super::PostgresSqlConstraintKind::Check),
            ] {
                let PostgresSqlAlterOperation::AddConstraint {
                    constraint,
                    not_valid,
                } = &constraints[index].2[0]
                else {
                    panic!()
                };
                assert_eq!(constraint.kind, expected);
                assert!(!not_valid);
            }
            for diagnostic in &block.diagnostics {
                let span = diagnostic.span.as_ref().unwrap();
                assert!(sql[span.start.offset..span.end.offset].starts_with("LOCK TABLE"));
            }
        }
        for statement in &all {
            assert_eq!(
                &sql[statement.span.start.offset..statement.span.end.offset],
                statement.sql
            );
            if let PostgresSqlStatementKind::Insert { insert } = &statement.facts {
                assert_eq!(insert.span.as_ref(), Some(&statement.span));
            }
        }
    }
}

#[test]
fn unsupported_nested_sql_projections_are_explicit_and_do_not_hide_supported_constraints() {
    let result = facts("nested-incomplete.sql");
    assert!(result.diagnostics.is_empty());
    let PostgresSqlStatementKind::DoBlock { block } = &result.statements[0].facts else {
        panic!()
    };
    assert!(!block.complete);
    assert_eq!(block.diagnostics.len(), 4, "{:?}", block.diagnostics);
    assert!(block
        .diagnostics
        .iter()
        .all(|diagnostic| diagnostic.span.is_some()));
    let mut all = Vec::new();
    occurrences(&result.statements[0], &mut all);
    assert!(all.iter().any(|statement| matches!(&statement.facts, PostgresSqlStatementKind::AlterTable { operations, .. } if matches!(&operations[0], PostgresSqlAlterOperation::AddConstraint { constraint, .. } if constraint.name.as_ref().is_some_and(|name| name.value == "supported")))));
}

#[test]
fn conditional_source_ranges_require_the_parser_owned_closing_if_token() {
    use super::super::conditional_source;
    use sqlparser::{dialect::PostgreSqlDialect, parser::Parser};
    let sql = fixture("conditional-branch.sql");
    let prepared = crate::codebase::postgres::parse::prepare_postgres_tokens(&sql);
    let mut parser = Parser::new(&PostgreSqlDialect {}).with_tokens_with_locations(prepared.tokens);
    let sqlparser::ast::Statement::If(mut value) = parser.parse_statement().unwrap() else {
        panic!()
    };
    let cursor = value.if_block.start_token.0.span.start;
    let tokens = (0..parser.index())
        .map(|index| parser.token_at(index))
        .collect::<Vec<_>>();
    value.end_token = None;
    assert!(conditional_source::statement_range(
        &sqlparser::ast::Statement::If(value),
        &tokens,
        cursor
    )
    .unwrap_err()
    .contains("closing IF"));
}
