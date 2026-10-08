use super::{facts, fixture};
use crate::codebase::postgres::source::{locations::Locations, PostgresSqlStatementKind};

fn assert_slice(sql: &str, span: &Option<super::super::PostgresSqlSpan>, expected: &str) {
    let span = span.as_ref().expect("parser-proven constraint source span");
    let newline = if sql.contains("\r\n") { "\r\n" } else { "\n" };
    assert_eq!(
        &sql[span.start.offset..span.end.offset],
        expected.replace('\n', newline)
    );
}

#[test]
fn create_and_alter_constraints_keep_exact_original_source_ranges() {
    let sql = fixture("constraint-source-spans.sql");
    let facts = facts("constraint-source-spans.sql");
    assert!(facts.diagnostics.is_empty(), "{:?}", facts.diagnostics);
    let PostgresSqlStatementKind::CreateTable {
        columns,
        constraints,
        ..
    } = &facts.statements[0].facts
    else {
        panic!()
    };
    assert_slice(&sql, &columns[0].constraints[0].span, "PRIMARY KEY");
    assert_slice(
        &sql,
        &columns[1].constraints[0].span,
        "REFERENCES public.parent(id)",
    );
    assert_slice(
        &sql,
        &columns[2].constraints[0].span,
        "REFERENCES public.parent(id) ON DELETE SET DEFAULT",
    );
    assert_slice(&sql, &columns[3].constraints[0].span, "UNIQUE");
    assert_slice(&sql, &columns[3].constraints[1].span, "UNIQUE");
    assert_slice(
        &sql,
        &columns[4].constraints[0].span,
        "PRIMARY /* split keyword */ KEY",
    );
    assert_slice(&sql, &columns[5].constraints[0].span, "UNIQUE");
    assert_slice(&sql, &columns[6].constraints[0].span, "UNIQUE");
    assert_slice(
        &sql,
        &constraints[0].span,
        "CONSTRAINT \"ck_name\" CHECK (length('λ') > 0)",
    );
    let unicode_span = constraints[0].span.as_ref().unwrap();
    assert_eq!((unicode_span.start.line, unicode_span.start.column), (9, 3));
    assert_eq!(
        unicode_span.end.offset - unicode_span.start.offset,
        "CONSTRAINT \"ck_name\" CHECK (length('λ') > 0)".len()
    );
    assert_slice(&sql, &constraints[1].span, "UNIQUE (parent_id)");
    let PostgresSqlStatementKind::AlterTable { operations, .. } = &facts.statements[1].facts else {
        panic!()
    };
    let crate::codebase::postgres::source::PostgresSqlAlterOperation::AddConstraint {
        constraint,
        not_valid,
    } = &operations[0]
    else {
        panic!()
    };
    assert!(*not_valid);
    assert_slice(
        &sql,
        &constraint.span,
        "CONSTRAINT fk_parent FOREIGN KEY (parent_id) REFERENCES public.parent(id) NOT VALID",
    );
    let crate::codebase::postgres::source::PostgresSqlAlterOperation::AddConstraint {
        constraint,
        ..
    } = &operations[1]
    else {
        panic!()
    };
    assert_slice(&sql, &constraint.span, "CHECK (id > 0)");
    let PostgresSqlStatementKind::AlterTable { operations, .. } = &facts.statements[2].facts else {
        panic!()
    };
    let crate::codebase::postgres::source::PostgresSqlAlterOperation::AddConstraint {
        constraint,
        ..
    } = &operations[1]
    else {
        panic!()
    };
    assert_slice(
        &sql,
        &constraint.span,
        "CONSTRAINT after_other_operation CHECK (parent_id > 0)",
    );
    let PostgresSqlStatementKind::AlterTable { operations, .. } = &facts.statements[3].facts else {
        panic!()
    };
    let crate::codebase::postgres::source::PostgresSqlAlterOperation::AddConstraint {
        constraint,
        ..
    } = &operations[1]
    else {
        panic!()
    };
    assert_slice(
        &sql,
        &constraint.span,
        "CONSTRAINT after_array_default CHECK (parent_id > 0)",
    );
    let PostgresSqlStatementKind::AlterTable { operations, .. } = &facts.statements[4].facts else {
        panic!()
    };
    let crate::codebase::postgres::source::PostgresSqlAlterOperation::AddColumn { column } =
        &operations[0]
    else {
        panic!()
    };
    assert_slice(&sql, &column.constraints[0].span, "CHECK (added > 0)");
    let PostgresSqlStatementKind::CreateTable { columns, .. } = &facts.statements[5].facts else {
        panic!()
    };
    for (column, expected) in [
        (0, "CONSTRAINT named_pk PRIMARY KEY"),
        (1, "CONSTRAINT named_unique UNIQUE"),
        (2, "CONSTRAINT named_fk REFERENCES public.parent(id)"),
        (3, "CONSTRAINT named_check CHECK (named_check > 0)"),
        (4, "UNIQUE"),
        (5, "UNIQUE"),
        (6, "CHECK (generated_after_check > 0)"),
    ] {
        assert_slice(&sql, &columns[column].constraints[0].span, expected);
    }
    assert!(columns[7].constraints[0].span.is_none());
}

#[test]
fn issue_example_constraint_ranges_are_exact_utf8_bytes() {
    let sql = fixture("constraint-source-spans-basic.sql");
    let facts = facts("constraint-source-spans-basic.sql");
    assert!(facts.diagnostics.is_empty(), "{:?}", facts.diagnostics);
    let PostgresSqlStatementKind::CreateTable {
        columns,
        constraints,
        ..
    } = &facts.statements[0].facts
    else {
        panic!()
    };
    let inline = columns[0].constraints[0].span.as_ref().unwrap();
    let check = constraints[0].span.as_ref().unwrap();
    let PostgresSqlStatementKind::AlterTable { operations, .. } = &facts.statements[1].facts else {
        panic!()
    };
    let crate::codebase::postgres::source::PostgresSqlAlterOperation::AddConstraint {
        constraint,
        ..
    } = &operations[0]
    else {
        panic!()
    };
    let alter_span = constraint.span.as_ref().unwrap();
    let newline = if sql.contains("\r\n") { "\r\n" } else { "\n" };
    for (span, expected) in [
        (inline, "REFERENCES public.parents(id)"),
        (check, "CONSTRAINT children_check CHECK (parent_id IS NOT NULL)"),
        (alter_span, "CONSTRAINT children_parent_fk\n  FOREIGN KEY (parent_id) REFERENCES public.parents(id) NOT VALID"),
    ] {
        let expected = expected.replace('\n', newline);
        assert_eq!(&sql[span.start.offset..span.end.offset], expected);
        assert_eq!(span.start.offset, sql.find(&expected).unwrap());
    }
}

#[test]
fn missing_prepared_tokens_never_produce_guessed_constraint_locations() {
    use crate::codebase::postgres::parse::prepare_postgres_tokens;
    use sqlparser::{ast::Statement, dialect::PostgreSqlDialect, parser::Parser};
    let sql = fixture("constraint-source-spans-unproven.sql");
    let statements = Parser::parse_sql(&PostgreSqlDialect {}, &sql).unwrap();
    let create_sql = &sql;
    let Statement::CreateTable(table) = &statements[0] else {
        panic!()
    };
    let locations = super::super::locations::Locations::new(create_sql);
    assert!(super::super::constraint_spans::table_span(
        &table.constraints[0],
        &[],
        &[],
        &locations
    )
    .is_none());
    assert_eq!(
        super::super::constraint_spans::column_span(&table.columns[0], &[], &[], &locations),
        [None]
    );
    let named_sql = fixture("constraint-source-spans.sql");
    let prepared = prepare_postgres_tokens(&named_sql);
    let statements = Parser::new(&PostgreSqlDialect {})
        .with_tokens_with_locations(prepared.tokens.clone())
        .parse_statements()
        .unwrap();
    let Statement::CreateTable(named_table) = statements.last().unwrap() else {
        panic!()
    };
    let ranges = super::super::constraint_spans::table_ranges(&prepared.tokens);
    let column_range = |column: &sqlparser::ast::ColumnDef| {
        ranges
            .iter()
            .copied()
            .find(|(start, end)| {
                prepared.tokens[*start..*end].iter().any(|token| {
                    token.span.start <= column.name.span.start
                        && column.name.span.start <= token.span.end
                })
            })
            .unwrap()
    };
    let primary_range = column_range(&named_table.columns[0]);
    let named_unique_range = column_range(&named_table.columns[1]);
    assert_eq!(
        super::super::constraint_spans::column_span_in_range(
            &named_table.columns[0],
            &prepared.tokens,
            (usize::MAX, usize::MAX),
            &Locations::new(&named_sql)
        ),
        [None]
    );
    assert_eq!(
        super::super::constraint_spans::column_span_in_range(
            &named_table.columns[0],
            &prepared.tokens,
            ranges[0],
            &Locations::new(&named_sql)
        ),
        [None]
    );
    assert_eq!(
        super::super::constraint_spans::column_span_in_range(
            &named_table.columns[0],
            &prepared.tokens,
            primary_range,
            &Locations::new("")
        ),
        [None]
    );

    let mut missing_primary = prepared.tokens.clone();
    missing_primary.retain(|token| {
        !(token.span.start >= prepared.tokens[primary_range.0].span.start
            && token.span.end <= prepared.tokens[primary_range.1 - 1].span.end
            && matches!(&token.token, sqlparser::tokenizer::Token::Word(word)
                if word.value.eq_ignore_ascii_case("PRIMARY")
                    || word.value.eq_ignore_ascii_case("KEY")))
    });
    assert_eq!(
        super::super::constraint_spans::column_span(
            &named_table.columns[0],
            &missing_primary,
            &super::super::constraint_spans::table_ranges(&missing_primary),
            &Locations::new(&named_sql)
        ),
        [None]
    );
    let mut missing_name_prefix = prepared.tokens.clone();
    missing_name_prefix.retain(|token| {
        !(token.span.start >= prepared.tokens[named_unique_range.0].span.start
            && token.span.end <= prepared.tokens[named_unique_range.1 - 1].span.end
            && matches!(&token.token, sqlparser::tokenizer::Token::Word(word)
                if word.value.eq_ignore_ascii_case("CONSTRAINT")))
    });
    assert_eq!(
        super::super::constraint_spans::column_span(
            &named_table.columns[1],
            &missing_name_prefix,
            &super::super::constraint_spans::table_ranges(&missing_name_prefix),
            &Locations::new(&named_sql)
        ),
        [None]
    );
    assert!(super::super::constraint_spans::table_ranges(&[]).is_empty());

    let Statement::AlterTable(alter) = &statements[1] else {
        panic!()
    };
    assert!(super::super::constraint_spans::alter_ranges(
        &[],
        sqlparser::tokenizer::Location::empty()
    )
    .is_empty());
    assert!(super::super::constraint_spans::alter_span(
        0,
        &alter.operations[0],
        &[],
        &[],
        &locations
    )
    .is_none());
    assert!(super::super::constraint_spans::alter_span(
        0,
        &alter.operations[0],
        &prepared.tokens,
        &[(usize::MAX, usize::MAX)],
        &locations
    )
    .is_none());
    assert!(super::super::constraint_spans::alter_span(
        0,
        &alter.operations[0],
        &prepared.tokens,
        &[(0, 1)],
        &locations
    )
    .is_none());
    let mut missing_constraint_keyword = prepared.tokens.clone();
    missing_constraint_keyword.retain(|token| {
        !matches!(&token.token, sqlparser::tokenizer::Token::Word(word)
            if ["CONSTRAINT", "PRIMARY", "UNIQUE", "FOREIGN", "CHECK"]
                .iter().any(|expected| word.value.eq_ignore_ascii_case(expected)))
    });
    assert!(super::super::constraint_spans::alter_span(
        0,
        &alter.operations[0],
        &missing_constraint_keyword,
        &[(0, missing_constraint_keyword.len())],
        &locations
    )
    .is_none());
    let Statement::AlterTable(other) = &statements[2] else {
        panic!()
    };
    assert!(super::super::constraint_spans::alter_span(
        0,
        &other.operations[0],
        &[],
        &[],
        &locations
    )
    .is_none());
}

#[test]
fn wrapper_and_literal_execute_constraints_keep_their_own_source_coordinates() {
    let sql = fixture("constraint-source-spans-nested.sql");
    let facts = facts("constraint-source-spans-nested.sql");
    assert!(facts.diagnostics.is_empty(), "{:?}", facts.diagnostics);
    let PostgresSqlStatementKind::Wrapper { wrapper } = &facts.statements[0].facts else {
        panic!()
    };
    let PostgresSqlStatementKind::CreateTable { constraints, .. } = &wrapper.statements[0].facts
    else {
        panic!()
    };
    let span = constraints[0].span.as_ref().unwrap();
    assert_eq!(
        &sql[span.start.offset..span.end.offset],
        "CONSTRAINT wrapper_check CHECK (id > 0)"
    );

    let PostgresSqlStatementKind::DoBlock { block } = &facts.statements[1].facts else {
        panic!()
    };
    let PostgresSqlStatementKind::LiteralExecute { execute } = &block.statements[0].facts else {
        panic!()
    };
    let PostgresSqlStatementKind::CreateTable { constraints, .. } = &execute.statements[0].facts
    else {
        panic!()
    };
    let span = constraints[0].span.as_ref().unwrap();
    assert_eq!(
        &execute.decoded_sql[span.start.offset..span.end.offset],
        "CONSTRAINT decoded_check CHECK (id > 0)"
    );
}
