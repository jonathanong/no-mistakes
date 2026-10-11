use super::{expression, Expr};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{Expression, TaggedTemplateExpression, TemplateLiteral};

pub(super) fn template(value: &TemplateLiteral<'_>, source: &str) -> Expr {
    let mut parts = Vec::new();
    for (index, quasi) in value.quasis.iter().enumerate() {
        if index > 0 {
            parts.push(expression(&value.expressions[index - 1], source));
        }
        parts.push(Expr::Text(
            quasi
                .value
                .cooked
                .as_ref()
                .expect("successful untagged templates have cooked text")
                .to_string(),
        ));
    }
    Expr::Template(parts)
}

pub(super) fn tagged(
    value: &TaggedTemplateExpression<'_>,
    _expr: &Expression<'_>,
    source: &str,
) -> Expr {
    let tag = match unwrap_ts_wrappers(&value.tag) {
        Expression::Identifier(tag) => tag.name.to_string(),
        Expression::StaticMemberExpression(member)
            if member.property.name == "raw"
                && matches!(unwrap_ts_wrappers(&member.object), Expression::Identifier(id) if id.name == "String") =>
        {
            "String.raw".into()
        }
        _ => String::new(),
    };
    let parts = if tag == "String.raw" {
        let mut parts = Vec::new();
        for (index, quasi) in value.quasi.quasis.iter().enumerate() {
            if index > 0 {
                parts.push(expression(&value.quasi.expressions[index - 1], source));
            }
            parts.push(Expr::Text(quasi.value.raw.to_string()));
        }
        parts
    } else {
        let mut parts = Vec::new();
        for (index, quasi) in value.quasi.quasis.iter().enumerate() {
            if index > 0 {
                parts.push(Expr::Text(format!("sql_placeholder_{index}")));
            }
            parts.push(Expr::Text(quasi.value.cooked.as_ref().map_or_else(
                || quasi.value.raw.to_string(),
                |value| value.to_string(),
            )));
        }
        parts
    };
    let mut effects = vec![expression(&value.tag, source)];
    if tag != "String.raw" {
        effects.extend(
            value
                .quasi
                .expressions
                .iter()
                .map(|expr| expression(expr, source)),
        );
    }
    Expr::Tagged(tag, parts, effects)
}
