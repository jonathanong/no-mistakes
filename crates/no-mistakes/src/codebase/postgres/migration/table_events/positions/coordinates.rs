use sqlparser::ast::DollarQuotedString;
use sqlparser::tokenizer::TokenWithSpan;

pub(super) fn start_line(code: &[&TokenWithSpan]) -> usize {
    code.first()
        .map_or(1, |token| token.span.start.line as usize)
}

pub(super) fn start_column(code: &[&TokenWithSpan]) -> usize {
    code.first()
        .map_or(0, |token| token.span.start.column as usize)
}

pub(super) fn body_position(
    outer: &TokenWithSpan,
    body: &DollarQuotedString,
    code: &[&TokenWithSpan],
) -> (usize, usize) {
    let line = start_line(code);
    let column = if line == 1 {
        outer.span.start.column as usize
            + body.tag.as_ref().map_or(2, |tag| tag.chars().count() + 2)
            + start_column(code)
            - 1
    } else {
        start_column(code)
    };
    (outer.span.start.line as usize + line - 1, column)
}
