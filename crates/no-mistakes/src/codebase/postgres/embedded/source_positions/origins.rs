use super::escapes;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::Expression;
use oxc_span::GetSpan;

/// One physical byte origin per recovered SQL byte, including decoded escapes.
pub(in crate::codebase::postgres::embedded) fn piece(
    raw: &str,
    decoded: &str,
    start: u32,
    raw_mode: bool,
) -> Vec<u32> {
    let mut out = Vec::with_capacity(decoded.len());
    let mut at = 0;
    let mut extra = 0;
    let mut origin = start;
    for character in decoded.chars() {
        if extra == 0 {
            if !raw_mode {
                while let Some(width) = escapes::continuation(&raw[at..]) {
                    at += width;
                }
            }
            origin = start + at as u32;
            let width = escapes::width(&raw[at..], raw_mode);
            extra = escapes::extra_characters(&raw[at..], width, raw_mode);
            at += width;
        } else {
            extra -= 1;
        }
        out.extend(std::iter::repeat_n(origin, character.len_utf8()));
    }
    out
}

pub(in crate::codebase::postgres::embedded) fn expression(
    expr: &Expression<'_>,
    source: &str,
    sql: &str,
) -> Vec<u32> {
    match unwrap_ts_wrappers(expr) {
        Expression::StringLiteral(literal) => piece(
            &source[literal.span.start as usize + 1..literal.span.end as usize - 1],
            sql,
            literal.span.start + 1,
            false,
        ),
        Expression::TemplateLiteral(template) if template.expressions.is_empty() => {
            let quasi = &template.quasis[0];
            piece(
                &source[quasi.span.start as usize..quasi.span.end as usize],
                sql,
                quasi.span.start,
                quasi.value.cooked.is_none(),
            )
        }
        Expression::TaggedTemplateExpression(tagged) if tagged.quasi.expressions.is_empty() => {
            let quasi = &tagged.quasi.quasis[0];
            piece(
                &source[quasi.span.start as usize..quasi.span.end as usize],
                sql,
                quasi.span.start,
                true,
            )
        }
        _ => vec![expr.span().start; sql.len()],
    }
}
