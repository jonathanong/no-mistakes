use super::{placeholders, tags};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{Expression, TemplateLiteral};
use std::collections::HashMap;

/// SQL text of a literal, tagged template, or template expression.
pub fn sql_text(expr: &Expression<'_>) -> Option<String> {
    unpublished_sql_text(expr).map(placeholders::publish_placeholders)
}

pub(in crate::codebase::postgres) fn unpublished_sql_text(expr: &Expression<'_>) -> Option<String> {
    match unwrap_ts_wrappers(expr) {
        Expression::StringLiteral(literal) => Some(crate::ast::js_literal(literal).into_owned()),
        Expression::TemplateLiteral(template) => Some(template_sql_text(template, false)),
        Expression::TaggedTemplateExpression(tagged) => {
            let use_raw = tags::is_string_raw_tag_spelling(&tagged.tag);
            Some(template_sql_text(&tagged.quasi, use_raw))
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

/// `use_raw` selects `String.raw`'s own runtime semantics (the literal
/// source characters, escape sequences un-processed) over the cooked form
/// every other tag — the trusted `sql` tag included — returns; see
/// [`tags::is_string_raw_tag_spelling`].
fn template_sql_text(template: &TemplateLiteral<'_>, use_raw: bool) -> String {
    let mut out = String::new();
    for (index, quasi) in template.quasis.iter().enumerate() {
        if index > 0 {
            out.push_str(&placeholders::internal_placeholder(index));
        }
        out.push_str(&tags::quasi_text(quasi, use_raw));
    }
    out
}
