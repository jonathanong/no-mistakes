use super::{ddl, locations::Locations, types::*};
use crate::codebase::postgres::parse::PreparedPostgresTokens;
use sqlparser::{ast::Statement, dialect::PostgreSqlDialect, parser::Parser, tokenizer::Token};

pub(super) fn collect(
    source: &PostgresSqlSource,
    prepared: PreparedPostgresTokens,
    locations: &Locations<'_>,
) -> PostgresSqlFacts {
    collect_program(source, prepared, locations, 0)
}

pub(super) fn collect_program(
    source: &PostgresSqlSource,
    mut prepared: PreparedPostgresTokens,
    locations: &Locations<'_>,
    depth: usize,
) -> PostgresSqlFacts {
    let mut result = PostgresSqlFacts {
        schema_version: 1,
        file_name: source.file_name.clone(),
        statements: Vec::new(),
        diagnostics: Vec::new(),
    };
    ddl::prepare_trigger_arguments(&mut prepared.tokens);
    let generated = super::generated::prepare(&mut prepared.tokens);
    let fetch_expressions =
        crate::codebase::postgres::parse::fetch_expression::prepare(&mut prepared.tokens);
    // Conditional AST parsing owns its nested statements. Keep that grammar
    // intact; unsupported partial conflict targets in procedural bodies diagnose.
    let conflict_markers = if depth == 0 {
        super::insert::parsing::prepare(&mut prepared.tokens)
    } else {
        Vec::new()
    };
    let mut parser = Parser::new(&PostgreSqlDialect {}).with_tokens_with_locations(prepared.tokens);
    let mut ordinal = 0;
    while parser.peek_token().token != Token::EOF {
        if parser.consume_token(&Token::SemiColon) {
            continue;
        }
        let start = parser.peek_token().span.start;
        let start_index = parser.index();
        let parsed = if super::procedural::starts(&parser) {
            super::procedural::collect(&mut parser, source, locations, depth)
        } else {
            super::insert::parsing::parse(&mut parser, &conflict_markers)
                .map_err(|error| error.to_string())
                .and_then(|(mut statement, conflict_predicate)| {
                    prepared
                        .recursive_views
                        .validate(&statement)
                        .map_err(|error| error.to_string())?;
                    prepared.recursive_views.restore(&mut statement);
                    crate::codebase::postgres::parse::fetch_expression::restore(
                        &mut statement,
                        &fetch_expressions,
                    );
                    super::generated::restore(
                        &mut statement,
                        &generated,
                        parser.token_at(parser.index().saturating_sub(1)).span.end,
                    );
                    if let Statement::Insert(value) = &statement {
                        return Ok(PostgresSqlStatementKind::Insert {
                            insert: Box::new(super::insert::project(
                                value,
                                conflict_predicate.as_ref(),
                                locations,
                            )),
                        });
                    }
                    if let Statement::If(value) = &mut statement {
                        if depth == 0 {
                            return Err("Conditional statements require a procedural body".into());
                        }
                        let tokens = (start_index..parser.index())
                            .map(|index| parser.token_at(index))
                            .collect::<Vec<_>>();
                        super::conditional::project(
                            value,
                            &tokens,
                            source,
                            locations,
                            &generated,
                            &prepared.recursive_views,
                        )
                    } else {
                        let tables =
                            crate::codebase::postgres::statements::TableTokenIndex::from_iter(
                                (start_index..parser.index()).map(|index| parser.token_at(index)),
                            );
                        Ok(super::projection::project(
                            &statement,
                            locations,
                            &tables,
                            &prepared.recursive_views,
                        ))
                    }
                })
        };
        let complete = matches!(parser.peek_token().token, Token::SemiColon | Token::EOF);
        // Move projected facts once; cloning a nested program here repeats its subtree.
        let (facts, error) = match parsed {
            Ok(facts) if complete => (Some(facts), None),
            Ok(_) => (
                None,
                Some(format!(
                    "Expected statement delimiter, found {}",
                    parser.peek_token().token
                )),
            ),
            Err(error) => (None, Some(error)),
        };
        if error.is_some() {
            recover(&mut parser, &conflict_markers);
        }
        let end = if parser.peek_token().token == Token::SemiColon {
            parser.next_token().span.end
        } else {
            (start_index..parser.index())
                .filter_map(|index| {
                    let token = parser.token_at(index);
                    (!matches!(token.token, Token::Whitespace(_))).then_some(token.span.end)
                })
                .filter(|end| end.line > 0)
                .max()
                .unwrap_or(start)
        };
        let end = crate::codebase::postgres::parse::fetch_expression::source_end(
            &fetch_expressions,
            start,
            end,
        );
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
        if let Some(mut facts) = facts {
            if let PostgresSqlStatementKind::Insert { insert } = &mut facts {
                insert.span = Some(span.clone());
            }
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

fn recover(parser: &mut Parser<'_>, markers: &[sqlparser::tokenizer::Location]) {
    loop {
        let token = parser.peek_token();
        let boundary =
            token.token == Token::SemiColon && markers.binary_search(&token.span.start).is_err();
        if boundary || token.token == Token::EOF {
            break;
        }
        // Synthetic conflict delimiters belong to this failed statement, not
        // to the next ordinal. Only an original semicolon ends recovery.
        parser.next_token();
    }
}
