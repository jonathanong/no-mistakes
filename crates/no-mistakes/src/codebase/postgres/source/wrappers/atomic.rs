use super::*;

pub(super) fn collect(
    function: &mut PostgresSqlFunction,
    parser: &mut Parser<'_>,
    context: &Context<'_, '_>,
) {
    let start = parser.index();
    parser.next_token();
    let atomic = parser.parse_keyword(Keyword::ATOMIC);
    let end = end(parser, start);
    let supported = atomic
        && function
            .language
            .as_ref()
            .is_some_and(|value| value.eq_ignore_ascii_case("sql"));
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
                    if matches!(&statement, Statement::CreateTable(table) if table.query.is_some())
                    {
                        wrapper.diagnostics.push(diagnostic(
                            "Function child CREATE TABLE AS query facts are unsupported",
                            None,
                        ));
                    }
                    let tokens = (child_start..parser.index())
                        .map(|index| parser.token_at(index).clone())
                        .collect::<Vec<_>>();
                    if let Some(child) = context.statement(
                        &statement,
                        &tokens,
                        child_ordinal,
                        1,
                        insert_facts.as_ref(),
                    ) {
                        wrapper.statements.push(child);
                    }
                }
                Err(error) => {
                    wrapper
                        .diagnostics
                        .push(diagnostic(&error.to_string(), None));
                    while parser.index() < end
                        && !matches!(parser.peek_token().token, Token::SemiColon | Token::EOF)
                    {
                        parser.next_token();
                    }
                }
                Ok(_) => {
                    wrapper.diagnostics.push(diagnostic(
                        "Function child crossed END or has no statement delimiter",
                        None,
                    ));
                    while parser.index() < end
                        && !matches!(parser.peek_token().token, Token::SemiColon | Token::EOF)
                    {
                        parser.next_token();
                    }
                }
            }
        }
    } else {
        while parser.peek_token().span.start != boundary && parser.peek_token().token != Token::EOF
        {
            parser.next_token();
        }
        wrapper.diagnostics.push(diagnostic(
            "Only SQL-language BEGIN ATOMIC bodies expose child source occurrences",
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

fn end(parser: &Parser<'_>, start: usize) -> usize {
    let mut depth: usize = 0;
    let mut index = start;
    loop {
        let token = parser.token_at(index);
        if token.token == Token::EOF {
            return index;
        }
        if keyword(&token.token, Keyword::BEGIN) || keyword(&token.token, Keyword::CASE) {
            depth += 1;
        } else if keyword(&token.token, Keyword::END) {
            depth = depth.saturating_sub(1);
            if depth == 0 {
                return index;
            }
        }
        index += 1;
    }
}
