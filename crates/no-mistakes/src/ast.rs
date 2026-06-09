use anyhow::{Context, Result};
use oxc_allocator::Allocator;
use oxc_ast::ast::{BinaryExpression, BinaryOperator, Expression, Program, TemplateLiteral};
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType, Span};
use std::path::Path;

pub fn with_program<T>(
    path: &Path,
    source: &str,
    analyze: impl for<'a> FnOnce(&'a Program<'a>, &'a str) -> T,
) -> Result<T> {
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(path)
        .with_context(|| format!("unsupported JavaScript/TypeScript file: {}", path.display()))?;
    let parsed = Parser::new(&allocator, source, source_type).parse();

    if parsed.panicked || !parsed.errors.is_empty() {
        let detail = parsed
            .errors
            .first()
            .map(|e| format!("{e:?}"))
            .unwrap_or("unknown error (parser panicked)".to_string());
        anyhow::bail!("failed to parse {}: {detail}", path.display());
    }

    Ok(analyze(&parsed.program, source))
}

pub fn span_text(source: &str, span: Span) -> &str {
    source
        .get(span.start as usize..span.end as usize)
        .unwrap_or_default()
}

pub fn template_literal_text(template: &TemplateLiteral<'_>, source: &str) -> String {
    let mut text = String::new();
    for (index, quasi) in template.quasis.iter().enumerate() {
        text.push_str(
            quasi
                .value
                .cooked
                .as_ref()
                .unwrap_or(&quasi.value.raw)
                .as_str(),
        );
        if let Some(expression) = template.expressions.get(index) {
            text.push_str("${");
            text.push_str(span_text(source, expression.span()));
            text.push('}');
        }
    }
    text
}

/// Fold a `+` string-concatenation chain into a single path string, emitting any
/// non-string operand as an unresolved `${...}` interpolation (mirroring
/// [`template_literal_text`]). For example `'/users/' + userId` yields
/// `/users/${userId}`, which downstream route matching treats as a single dynamic segment.
///
/// Returns `None` when the top-level operator is not `+`, so callers can fall back to their
/// default handling for unrelated binary expressions.
pub fn binary_concat_path_text(expression: &BinaryExpression<'_>, source: &str) -> Option<String> {
    if expression.operator != BinaryOperator::Addition {
        return None;
    }
    let mut text = String::new();
    append_concat_operand(&mut text, &expression.left, source);
    append_concat_operand(&mut text, &expression.right, source);
    Some(text)
}

fn append_concat_operand(text: &mut String, expression: &Expression<'_>, source: &str) {
    match expression {
        Expression::StringLiteral(literal) => text.push_str(literal.value.as_str()),
        Expression::TemplateLiteral(template) => {
            text.push_str(&template_literal_text(template, source))
        }
        Expression::BinaryExpression(binary) if binary.operator == BinaryOperator::Addition => {
            append_concat_operand(text, &binary.left, source);
            append_concat_operand(text, &binary.right, source);
        }
        Expression::ParenthesizedExpression(parenthesized) => {
            append_concat_operand(text, &parenthesized.expression, source)
        }
        other => {
            text.push_str("${");
            text.push_str(span_text(source, other.span()));
            text.push('}');
        }
    }
}

pub fn expression_path(expression: &Expression<'_>) -> Option<Vec<String>> {
    match expression {
        Expression::Identifier(identifier) => Some(vec![identifier.name.to_string()]),
        Expression::StaticMemberExpression(member) => {
            let mut parts = expression_path(&member.object).unwrap_or_default();
            parts.push(member.property.name.to_string());
            Some(parts)
        }
        Expression::ParenthesizedExpression(parenthesized) => {
            expression_path(&parenthesized.expression)
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests;
