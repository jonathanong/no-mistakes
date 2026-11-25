use super::{Context, PostgresSqlSpan, Span, Token, TokenWithSpan};

pub(super) fn span(tokens: &[TokenWithSpan], context: &Context<'_, '_>) -> Option<PostgresSqlSpan> {
    let mut significant = tokens
        .iter()
        .filter(|token| !matches!(token.token, Token::Whitespace(_)));
    let first = significant.next()?;
    let last = significant.next_back().unwrap_or(first);
    let original_end = tokens
        .iter()
        .filter_map(|token| {
            context
                .comments
                .borrow()
                .get(&token.span.start)
                .map(|comment| comment.end)
        })
        .max()
        .unwrap_or(last.span.end)
        .max(last.span.end);
    context.locations.span(Span {
        start: first.span.start,
        end: crate::codebase::postgres::parse::fetch_expression::source_end(
            context.fetch,
            first.span.start,
            original_end,
        ),
    })
}
