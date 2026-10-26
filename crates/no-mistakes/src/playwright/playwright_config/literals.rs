use crate::playwright::ast;
use anyhow::Result;
use oxc_ast::ast::Expression;

pub fn required_string(expression: &Expression<'_>, source: &str, name: &str) -> Result<String> {
    optional_string(expression, source)
        .ok_or_else(|| anyhow::anyhow!("expected string literal for {name}"))
}

pub fn optional_string(expression: &Expression<'_>, source: &str) -> Option<String> {
    match expression {
        Expression::StringLiteral(literal) => Some(crate::ast::js_literal(&literal).into_owned()),
        Expression::TemplateLiteral(template) if template.expressions.is_empty() => {
            Some(ast::template_literal_text(template, source))
        }
        Expression::ParenthesizedExpression(parenthesized) => {
            optional_string(&parenthesized.expression, source)
        }
        _ => None,
    }
}
