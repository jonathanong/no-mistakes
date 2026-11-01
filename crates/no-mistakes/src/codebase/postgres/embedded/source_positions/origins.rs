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
                while let Some(width) = raw.get(at..).and_then(escapes::continuation) {
                    at += width;
                }
            }
            let Some(rest) = raw.get(at..).filter(|rest| !rest.is_empty()) else {
                out.extend(std::iter::repeat_n(origin, character.len_utf8()));
                continue;
            };
            origin = start + at as u32;
            let width = escapes::width(rest, raw_mode);
            extra = escapes::extra_characters(rest, width, raw_mode);
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
        Expression::TemplateLiteral(template) => template_origins(template, source, false),
        Expression::TaggedTemplateExpression(tagged) => template_origins(
            &tagged.quasi,
            source,
            super::super::tags::is_string_raw_tag_spelling(&tagged.tag),
        ),
        _ => vec![expr.span().start; sql.len()],
    }
}

fn template_origins(
    template: &oxc_ast::ast::TemplateLiteral<'_>,
    source: &str,
    raw_mode: bool,
) -> Vec<u32> {
    let mut origins = Vec::new();
    for (index, quasi) in template.quasis.iter().enumerate() {
        if index > 0 {
            let marker = super::super::placeholders::internal_placeholder(index);
            origins.extend(std::iter::repeat_n(
                template.expressions[index - 1].span().start,
                marker.len(),
            ));
        }
        let raw = &source[quasi.span.start as usize..quasi.span.end as usize];
        let decoded = super::super::tags::quasi_text(quasi, raw_mode);
        origins.extend(piece(
            raw,
            &decoded,
            quasi.span.start,
            raw_mode || quasi.value.cooked.is_none(),
        ));
    }
    origins
}
