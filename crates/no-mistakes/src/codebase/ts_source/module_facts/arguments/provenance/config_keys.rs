//! Computed object keys that may be `unstubEnvs` without spelling it as a literal.
use super::super::PathProvenance;
use oxc_ast::ast::*;

pub(super) fn from_property(property: &ObjectProperty<'_>, out: &mut PathProvenance) {
    if !property.computed {
        return;
    }
    match property
        .key
        .as_expression()
        .and_then(literal_name)
        .as_deref()
    {
        Some("unstubEnvs" | "unstubAllEnvs") => out.env_unknown_write = true,
        Some(_) => {}
        None => out.opaque_config_key = true,
    }
}

fn literal_name(expression: &Expression<'_>) -> Option<String> {
    match expression.get_inner_expression() {
        Expression::StringLiteral(text) => Some(crate::ast::js_literal(text).into_owned()),
        Expression::TemplateLiteral(template) if template.expressions.is_empty() => {
            let [quasi] = template.quasis.as_slice() else {
                return None;
            };
            quasi
                .value
                .cooked
                .as_ref()
                .and_then(|text| text.as_str())
                .map(str::to_string)
        }
        Expression::NumericLiteral(_)
        | Expression::BooleanLiteral(_)
        | Expression::NullLiteral(_)
        | Expression::BigIntLiteral(_)
        | Expression::RegExpLiteral(_) => Some(String::new()),
        _ => None,
    }
}
