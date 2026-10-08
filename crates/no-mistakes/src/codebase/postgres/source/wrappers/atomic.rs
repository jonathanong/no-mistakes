use super::*;
mod boundary;
use boundary::end;

pub(super) fn collect(
    function: &mut PostgresSqlFunction,
    parser: &mut Parser<'_>,
    context: &Context<'_, '_>,
    depth: usize,
    sql_language: bool,
) {
    let start = parser.index();
    parser.next_token();
    let atomic = parser.parse_keyword(Keyword::ATOMIC);
    let end = end(parser, start);
    let supported = atomic && depth < 16 && sql_language;
    let mut wrapper = empty(
        PostgresSqlWrapperKind::FunctionDeclaration,
        PostgresSqlExecution::NonExecuting,
    );
    let boundary = parser.token_at(end).span.start;
    let mut ordinal = 0;
    if supported {
        while parser.peek_token().span.start != boundary && parser.peek_token().token != Token::EOF
        {
            if parser.consume_token(&Token::SemiColon) {
                continue;
            }
            let child_start = parser.index();
            let child_ordinal = ordinal;
            ordinal += 1;
            let parsed = super::super::insert::parsing::parse(parser, context.markers);
            let nested = atomic_child::function(
                parsed.as_ref().ok().map(|value| &value.0),
                parser,
                context,
                depth,
            );
            let overrun = parser.index() > end;
            if overrun {
                while parser.index() > end {
                    parser.prev_token();
                }
            }
            match parsed {
                Ok((mut statement, insert_facts))
                    if !overrun && parser.peek_token().token == Token::SemiColon =>
                {
                    if let Err(error) = context.restore(
                        &mut statement,
                        parser.token_at(parser.index().saturating_sub(1)).span.end,
                    ) {
                        wrapper.diagnostics.push(diagnostic(&error, None));
                        continue;
                    }
                    parser.next_token();
                    if let Some(message) = semantics::child_gap(&statement) {
                        wrapper.diagnostics.push(diagnostic(message, None));
                    }
                    let tokens = (child_start..parser.index())
                        .map(|index| parser.token_at(index).clone())
                        .collect::<Vec<_>>();
                    match context.statement(
                        &statement,
                        &tokens,
                        child_ordinal,
                        1,
                        insert_facts.as_ref(),
                    ) {
                        Ok(mut child) => {
                            if let Some(function) = nested {
                                child.facts = PostgresSqlStatementKind::CreateFunction { function };
                                finalize(&mut child.facts, &child.span);
                            }
                            wrapper.statements.push(child);
                        }
                        Err(error) => wrapper
                            .diagnostics
                            .push(diagnostic(&error, span(&tokens, context))),
                    }
                }
                Err(error) => {
                    wrapper
                        .diagnostics
                        .push(diagnostic(&error.to_string(), None));
                    atomic_child::recover(parser, end, context.markers);
                }
                Ok(_) => {
                    wrapper.diagnostics.push(diagnostic(
                        "Function child crossed END or has no statement delimiter",
                        None,
                    ));
                    atomic_child::recover(parser, end, context.markers);
                }
            }
        }
    } else {
        while parser.peek_token().span.start != boundary && parser.peek_token().token != Token::EOF
        {
            parser.next_token();
        }
        wrapper.diagnostics.push(diagnostic(
            if depth >= 16 {
                "Function declaration nesting exceeds the safety limit"
            } else {
                "Only SQL-language BEGIN ATOMIC bodies expose child source occurrences"
            },
            None,
        ));
    }
    let closed = keyword(&parser.peek_token().token, Keyword::END);
    if closed {
        parser.next_token();
    } else {
        wrapper.diagnostics.push(diagnostic(
            "Function BEGIN ATOMIC body has no authoritative END",
            None,
        ));
    }
    let tokens = (start..parser.index())
        .map(|index| parser.token_at(index).clone())
        .collect::<Vec<_>>();
    wrapper.span = span(&tokens, context);
    function.body_sql = wrapper
        .span
        .as_ref()
        .map(|span| context.source.sql[span.start.offset..span.end.offset].to_string());
    wrapper.complete = supported
        && closed
        && wrapper.diagnostics.is_empty()
        && wrapper
            .statements
            .iter()
            .all(|statement| complete(&statement.facts));
    if !wrapper.complete && wrapper.diagnostics.is_empty() {
        wrapper.diagnostics.push(diagnostic(
            "Function body contains incomplete child source facts",
            wrapper.span.clone(),
        ));
    }
    function.wrapper = wrapper;
}

/// Conflicting bodies remain one incomplete declaration and preserve the AS body.
pub(super) fn discard(parser: &mut Parser<'_>) {
    let boundary = parser.token_at(end(parser, parser.index())).span.start;
    while parser.peek_token().span.start != boundary && parser.peek_token().token != Token::EOF {
        parser.next_token();
    }
    if keyword(&parser.peek_token().token, Keyword::END) {
        parser.next_token();
    }
}
