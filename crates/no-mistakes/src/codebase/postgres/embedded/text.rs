use super::{placeholders, tags};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{Expression, TemplateLiteral, UnaryOperator};
use oxc_syntax::number::ToJsString;
use std::collections::HashMap;

/// SQL text of a literal, tagged template, or template expression.
pub fn sql_text(expr: &Expression<'_>) -> Option<String> {
    unpublished_sql_text(expr).map(placeholders::publish_placeholders)
}

pub(in crate::codebase::postgres) fn unpublished_sql_text(expr: &Expression<'_>) -> Option<String> {
    match unwrap_ts_wrappers(expr) {
        Expression::StringLiteral(literal) => Some(crate::ast::js_literal(literal).into_owned()),
        Expression::TemplateLiteral(template) => Some(template_sql_text(template, false, false)),
        Expression::TaggedTemplateExpression(tagged) => {
            let use_raw = tags::is_string_raw_tag_spelling(&tagged.tag);
            Some(template_sql_text(&tagged.quasi, use_raw, true))
        }
        _ => None,
    }
}

/// `sql_text` plus identifier lookup of in-scope SQL bindings.
pub fn executed_query_text(
    expr: &Expression<'_>,
    bindings: &HashMap<String, String>,
) -> Option<String> {
    if let Some(text) = sql_text(expr) {
        return Some(text);
    }
    match unwrap_ts_wrappers(expr) {
        Expression::Identifier(ident) => bindings.get(ident.name.as_str()).cloned(),
        _ => None,
    }
}

/// One hole in a template. `space` is inserted only between adjacent value
/// holes of a tagged template.
pub(in crate::codebase::postgres) struct RenderedHole {
    pub space: bool,
    pub text: String,
}

pub(in crate::codebase::postgres) fn render_hole(
    previous: &str,
    hole: &Expression<'_>,
    index: usize,
    already: &str,
    separate_adjacent: bool,
) -> RenderedHole {
    if previous.ends_with('$') {
        if let Some(digits) = positional_digits(hole, index) {
            return RenderedHole {
                space: false,
                text: digits,
            };
        }
    }
    RenderedHole {
        space: separate_adjacent && !already.is_empty() && previous.is_empty(),
        text: placeholders::internal_placeholder(index),
    }
}

fn positional_digits(hole: &Expression<'_>, index: usize) -> Option<String> {
    let number = match unwrap_ts_wrappers(hole) {
        Expression::NumericLiteral(literal) => literal.value,
        Expression::UnaryExpression(unary)
            if matches!(
                unary.operator,
                UnaryOperator::UnaryPlus | UnaryOperator::UnaryNegation
            ) =>
        {
            let Expression::NumericLiteral(literal) = unwrap_ts_wrappers(&unary.argument) else {
                return Some(index.to_string());
            };
            match unary.operator {
                UnaryOperator::UnaryNegation => -literal.value,
                _ => literal.value,
            }
        }
        Expression::TemplateLiteral(_) | Expression::TaggedTemplateExpression(_) => return None,
        expr if expr.is_literal() => return None,
        _ => return Some(index.to_string()),
    };
    let digits = number.to_js_string();
    digits
        .bytes()
        .all(|byte| byte.is_ascii_digit())
        .then_some(digits)
}

/// `use_raw` selects `String.raw`'s own runtime semantics (the literal
/// source characters, escape sequences un-processed) over the cooked form
/// every other tag — the trusted `sql` tag included — returns; see
/// [`tags::is_string_raw_tag_spelling`]. `separate_adjacent` is set only for
/// a tagged template, so a plain string builder does not split `$${n}`.
fn template_sql_text(
    template: &TemplateLiteral<'_>,
    use_raw: bool,
    separate_adjacent: bool,
) -> String {
    let mut out = String::new();
    for (index, quasi) in template.quasis.iter().enumerate() {
        if index > 0 {
            let previous = tags::quasi_text(&template.quasis[index - 1], use_raw);
            let rendered = render_hole(
                &previous,
                &template.expressions[index - 1],
                index,
                &out,
                separate_adjacent,
            );
            if rendered.space {
                out.push(' ');
            }
            out.push_str(&rendered.text);
        }
        out.push_str(&tags::quasi_text(quasi, use_raw));
    }
    out
}
