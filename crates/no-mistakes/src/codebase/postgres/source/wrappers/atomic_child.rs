use super::*;

/// A nested declaration continues on the same prepared parser and token stream.
pub(super) fn function(
    statement: Option<&Statement>,
    parser: &mut Parser<'_>,
    context: &Context<'_, '_>,
    depth: usize,
) -> Option<PostgresSqlFunction> {
    let Some(Statement::CreateFunction(value)) = statement else {
        return None;
    };
    if !keyword(&parser.peek_token().token, Keyword::BEGIN) {
        return None;
    }
    Some(collect(value, parser, context, depth + 1))
}

/// Synthetic conflict delimiters never end an original child occurrence.
pub(super) fn recover(
    parser: &mut Parser<'_>,
    end: usize,
    markers: &[sqlparser::tokenizer::Location],
) {
    while parser.index() < end && parser.peek_token().token != Token::EOF {
        let token = parser.peek_token();
        if token.token == Token::SemiColon && markers.binary_search(&token.span.start).is_err() {
            break;
        }
        parser.next_token();
    }
}

pub(super) fn collect(
    value: &sqlparser::ast::CreateFunction,
    parser: &mut Parser<'_>,
    context: &Context<'_, '_>,
    depth: usize,
) -> PostgresSqlFunction {
    let mut function = super::super::ddl::function(value, context.locations);
    if keyword(&parser.peek_token().token, Keyword::BEGIN) {
        if value.function_body.is_some() {
            atomic::discard(parser);
            function.wrapper.diagnostics.push(diagnostic(
                "Conflicting function bodies are unsupported; original AS body retained",
                None,
            ));
        } else {
            let sql_language = value.language.as_ref().is_none_or(|language| {
                language.value == "sql"
                    || language.quote_style.is_none() && language.value.eq_ignore_ascii_case("sql")
            });
            atomic::collect(&mut function, parser, context, depth, sql_language);
        }
    }
    function
}
