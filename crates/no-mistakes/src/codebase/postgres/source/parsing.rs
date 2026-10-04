use super::{columns, ddl, expressions::name, indexes, locations::Locations, types::*};
use crate::codebase::postgres::parse::PreparedPostgresTokens;
use sqlparser::{ast::Statement, dialect::PostgreSqlDialect, parser::Parser, tokenizer::Token};

pub(super) fn collect(
    source: &PostgresSqlSource,
    mut prepared: PreparedPostgresTokens,
    locations: &Locations<'_>,
) -> PostgresSqlFacts {
    let mut result = PostgresSqlFacts {
        schema_version: 1,
        file_name: source.file_name.clone(),
        statements: Vec::new(),
        diagnostics: Vec::new(),
    };
    ddl::prepare_trigger_arguments(&mut prepared.tokens);
    let mut parser = Parser::new(&PostgreSqlDialect {}).with_tokens_with_locations(prepared.tokens);
    let mut ordinal = 0;
    while parser.peek_token().token != Token::EOF {
        if parser.consume_token(&Token::SemiColon) {
            continue;
        }
        let start = parser.peek_token().span.start;
        let start_index = parser.index();
        let parsed = parser.parse_statement();
        let complete = matches!(parser.peek_token().token, Token::SemiColon | Token::EOF);
        let facts = parsed.as_ref().ok().filter(|_| complete).map(|statement| {
            let tables = crate::codebase::postgres::statements::TableTokenIndex::from_iter(
                (start_index..parser.index()).map(|index| parser.token_at(index)),
            );
            project(statement, locations, &tables)
        });
        let error = parsed.err().map(|error| error.to_string()).or_else(|| {
            (!complete).then(|| {
                format!(
                    "Expected statement delimiter, found {}",
                    parser.peek_token().token
                )
            })
        });
        if error.is_some() {
            while !matches!(parser.peek_token().token, Token::SemiColon | Token::EOF) {
                parser.next_token();
            }
        }
        let end = if parser.peek_token().token == Token::SemiColon {
            parser.next_token().span.end
        } else {
            parser.token_at(parser.index().saturating_sub(1)).span.end
        };
        let Some(span) = locations.span(sqlparser::tokenizer::Span { start, end }) else {
            // Parser compatibility rewrites can introduce synthetic token positions.
            // Do not expose a fabricated source slice when their boundary is unmappable.
            result.diagnostics.push(PostgresSqlDiagnostic {
                message: "Statement source boundary is unavailable after parser compatibility normalization".into(),
                span: locations.position(start).map(|position| locations.range(position.offset, position.offset)),
            });
            ordinal += 1;
            continue;
        };
        if let Some(facts) = facts {
            if matches!(&facts, PostgresSqlStatementKind::CreateView { view } if !view.dependencies_complete)
            {
                result.diagnostics.push(PostgresSqlDiagnostic { message: "View TABLE-arm dependencies have ambiguous source identity; dependencies are incomplete".into(), span: Some(span.clone()) });
            }
            result.statements.push(PostgresSqlStatement {
                ordinal,
                sql: source.sql[span.start.offset..span.end.offset].to_string(),
                span,
                facts,
            });
        } else if let Some(message) = error {
            result.diagnostics.push(PostgresSqlDiagnostic {
                message,
                span: Some(span),
            });
        }
        ordinal += 1;
    }
    if let Some(error) = prepared.lexical_error {
        result.diagnostics.push(PostgresSqlDiagnostic {
            message: error.to_string(),
            span: locations
                .position(error.location)
                .map(|position| locations.range(position.offset, source.sql.len())),
        });
    }
    result
}

fn project(
    statement: &Statement,
    locations: &Locations<'_>,
    tables: &crate::codebase::postgres::statements::TableTokenIndex,
) -> PostgresSqlStatementKind {
    if let Some(drop) = super::drop_facts::drop(statement) {
        return PostgresSqlStatementKind::Drop { drop };
    }
    match statement {
        Statement::CreateTable(value) => PostgresSqlStatementKind::CreateTable {
            table: name(&value.name),
            columns: value
                .columns
                .iter()
                .map(|column| columns::column(column, locations))
                .collect(),
            constraints: value
                .constraints
                .iter()
                .map(|constraint| columns::table_constraint(constraint, locations))
                .collect(),
            temporary: value.temporary,
        },
        Statement::AlterTable(value) => PostgresSqlStatementKind::AlterTable {
            table: name(&value.name),
            operations: value
                .operations
                .iter()
                .map(|operation| super::alter::alter(operation, locations))
                .collect(),
        },
        Statement::CreateIndex(value) => PostgresSqlStatementKind::CreateIndex {
            index: indexes::index(value, locations),
        },
        Statement::CreateView(value) => PostgresSqlStatementKind::CreateView {
            view: ddl::view(value, locations, tables),
        },
        Statement::CreateTrigger(value) => PostgresSqlStatementKind::CreateTrigger {
            trigger: ddl::trigger(value, locations),
        },
        Statement::CreateFunction(value) => PostgresSqlStatementKind::CreateFunction {
            function: ddl::function(value, locations),
        },
        _ => PostgresSqlStatementKind::Other,
    }
}
