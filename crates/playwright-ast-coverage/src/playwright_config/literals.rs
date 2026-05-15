use crate::ast;
use anyhow::Result;
use oxc_ast::ast::{ArrayExpression, ArrayExpressionElement, Expression};

pub fn required_string(expression: &Expression<'_>, source: &str, name: &str) -> Result<String> {
    optional_string(expression, source)
        .ok_or_else(|| anyhow::anyhow!("expected string literal for {name}"))
}

pub fn optional_string(expression: &Expression<'_>, source: &str) -> Option<String> {
    match expression {
        Expression::StringLiteral(literal) => Some(literal.value.to_string()),
        Expression::TemplateLiteral(template) if template.expressions.is_empty() => {
            Some(ast::template_literal_text(template, source))
        }
        Expression::ParenthesizedExpression(parenthesized) => {
            optional_string(&parenthesized.expression, source)
        }
        _ => None,
    }
}

pub fn required_string_or_array(
    expression: &Expression<'_>,
    source: &str,
    name: &str,
) -> Result<Vec<String>> {
    if let Some(value) = optional_string(expression, source) {
        return Ok(vec![value]);
    }
    let Some(Expression::ArrayExpression(array)) = parenthesized_expression(expression) else {
        anyhow::bail!("expected string literal or string array for {name}");
    };
    string_array(array, source, name)
}

pub fn parenthesized_expression<'a>(expression: &'a Expression<'a>) -> Option<&'a Expression<'a>> {
    match expression {
        Expression::ParenthesizedExpression(parenthesized) => {
            parenthesized_expression(&parenthesized.expression)
        }
        _ => Some(expression),
    }
}

fn string_array(array: &ArrayExpression<'_>, source: &str, name: &str) -> Result<Vec<String>> {
    let mut values = Vec::new();
    let mut saw_regex = false;
    let mut saw_unsupported = false;
    for element in &array.elements {
        match element {
            ArrayExpressionElement::StringLiteral(literal) => {
                values.push(literal.value.to_string())
            }
            ArrayExpressionElement::TemplateLiteral(template)
                if template.expressions.is_empty() =>
            {
                values.push(ast::template_literal_text(template, source));
            }
            ArrayExpressionElement::RegExpLiteral(_) => saw_regex = true,
            _ => saw_unsupported = true,
        }
    }
    if saw_regex {
        anyhow::bail!("regular-expression {name} patterns are not supported; use string globs");
    }
    if saw_unsupported || values.is_empty() {
        anyhow::bail!("expected string literal or string array for {name}");
    }
    Ok(values)
}
