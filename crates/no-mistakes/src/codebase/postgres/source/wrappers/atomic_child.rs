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
    let mut function = super::super::ddl::function(value, context.locations);
    atomic::collect(&mut function, parser, context, depth + 1);
    Some(function)
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
