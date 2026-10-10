use super::{ddl, locations::Locations, types::*};
use crate::codebase::postgres::parse::PreparedPostgresTokens;
use sqlparser::{ast::Statement, parser::Parser, tokenizer::Token};

pub(super) fn collect_program(
    source: &PostgresSqlSource,
    mut prepared: PreparedPostgresTokens,
    locations: &Locations<'_>,
    depth: usize,
    procedural: bool,
) -> PostgresSqlFacts {
    let mut result = PostgresSqlFacts {
        schema_version: 1,
        file_name: source.file_name.clone(),
        statements: Vec::new(),
        diagnostics: Vec::new(),
    };
    super::adjacent_strings::prepare(&mut prepared.tokens);
    let create_index_only = super::index_only::prepare(&mut prepared.tokens);
    super::wrappers::prepare(&mut prepared.tokens);
    ddl::prepare_trigger_arguments(&mut prepared.tokens);
    let generated = super::generated::prepare(&mut prepared.tokens);
    let fetch_expressions =
        crate::codebase::postgres::parse::fetch_expression::prepare(&mut prepared.tokens);
    // Conditional AST parsing owns its nested statements. Keep that grammar
    // intact; unsupported partial conflict targets in procedural bodies diagnose.
    let conflict_markers = if !procedural {
        super::insert::parsing::prepare(&mut prepared.tokens)
    } else {
        Vec::new()
    };
    let executes = super::execute_preparation::prepare(
        &mut prepared.tokens,
        source,
        locations,
        depth,
        procedural,
    );
    let parser = Parser::new(&super::dialect::PostgresSourceDialect)
        .with_tokens_with_locations(prepared.tokens);
    let (mut parser, comments) = super::metadata_preparation::prepare(parser, locations);
    let wrapper_context = super::wrappers::Context::new(
        source,
        locations,
        &prepared.recursive_views,
        &fetch_expressions,
        &generated,
        super::wrappers::child::Markers {
            inserts: &conflict_markers,
            index_only: create_index_only,
        },
        comments,
    )
    .with_executes(executes);
    let mut ordinal = 0;
    while parser.peek_token().token != Token::EOF {
        if parser.consume_token(&Token::SemiColon) {
            continue;
        }
        let start = parser.peek_token().span.start;
        let start_index = parser.index();
        let parsed = if let Some(facts) = wrapper_context.take_prepared(start) {
            super::recovery::recover(&mut parser, &[]);
            facts
        } else if super::procedural::starts(&parser) {
            super::procedural::collect(&mut parser, source, locations, depth)
        } else if super::metadata::starts(&parser) {
            super::metadata::collect(&mut parser, locations)
        } else if super::wrappers::starts(&parser) {
            Ok(super::wrappers::collect(&mut parser, &wrapper_context))
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
                    if let Statement::CreateFunction(value) = &statement {
                        return Ok(PostgresSqlStatementKind::CreateFunction {
                            function: super::wrappers::function(
                                value,
                                &mut parser,
                                &wrapper_context,
                            ),
                        });
                    }
                    if let Statement::Insert(value) = &statement {
                        return Ok(super::insert::source_projection::project_insert(
                            value,
                            conflict_predicate.as_ref(),
                            locations,
                            &parser,
                            start_index,
                        ));
                    }
                    if let Statement::If(value) = &mut statement {
                        if !procedural {
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
                            &wrapper_context,
                        )
                    } else {
                        Ok(super::projection::project_parsed(
                            &statement,
                            &parser,
                            start_index..parser.index(),
                            locations,
                            &prepared.recursive_views,
                            super::index_only::contains(&wrapper_context.index_only, start),
                        ))
                    }
                })
        };
        // A lexical failure can leave a valid-looking statement prefix at EOF.
        // The missing tail belongs to that occurrence, never a successful fact.
        let complete = matches!(parser.peek_token().token, Token::SemiColon | Token::EOF)
            && !(parser.peek_token().token == Token::EOF && prepared.lexical_error.is_some());
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
            super::recovery::recover(&mut parser, &conflict_markers);
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
            wrapper_context.source_end(start, end),
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
            super::wrappers::finalize(&mut facts, &span);
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
    super::diagnostics::append_lexical_error(
        &mut result.diagnostics,
        prepared.lexical_error,
        locations,
        source.sql.len(),
    );
    result
}
