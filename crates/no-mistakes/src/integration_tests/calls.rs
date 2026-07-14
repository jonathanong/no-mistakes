use super::types::CallTarget;
use crate::ast;
use oxc_ast::ast::{Argument, ArrowFunctionExpression, CallExpression, Expression, Function};
use oxc_ast_visit::Visit;
use oxc_span::Span;

mod visitor;
use visitor::CallCollector;

pub(super) fn callback_argument<'a>(
    call: &'a CallExpression<'a>,
) -> Option<(&'a Argument<'a>, Span)> {
    call.arguments
        .iter()
        .rev()
        .find_map(|argument| match argument {
            Argument::ArrowFunctionExpression(arrow) => Some((argument, arrow.span)),
            Argument::FunctionExpression(function) => Some((argument, function.span)),
            _ => None,
        })
}

pub(super) fn test_name(call: &CallExpression<'_>) -> Option<String> {
    let path = callee_path(call)?;
    let first = path.first()?.as_str();
    if first != "test" && first != "it" {
        return None;
    }
    if path.iter().any(|part| part == "describe") {
        return None;
    }
    if path.iter().any(|part| part == "skip" || part == "fixme") {
        return None;
    }
    call.arguments.first().and_then(string_arg)
}

pub(super) fn describe_name(call: &CallExpression<'_>) -> Option<String> {
    let path = callee_path(call)?;
    if is_skipped_describe_path(&path) {
        return None;
    }
    let first = path.first()?.as_str();
    if first != "describe" && !(first == "test" && path.iter().any(|part| part == "describe")) {
        return None;
    }
    call.arguments.first().and_then(string_arg)
}

pub(super) fn is_skipped_describe(call: &CallExpression<'_>) -> bool {
    callee_path(call).is_some_and(|path| is_skipped_describe_path(&path))
}

fn callee_path(call: &CallExpression<'_>) -> Option<Vec<String>> {
    match &call.callee {
        Expression::CallExpression(inner) => callee_path(inner),
        callee => ast::expression_path(callee),
    }
}

fn is_skipped_describe_path(path: &[String]) -> bool {
    let first = path.first().map(String::as_str);
    let is_describe = first == Some("describe")
        || (first == Some("test") && path.iter().any(|part| part == "describe"));
    is_describe && path.iter().any(|part| part == "skip" || part == "fixme")
}

fn string_arg(argument: &Argument<'_>) -> Option<String> {
    match argument {
        Argument::StringLiteral(literal) => Some(literal.value.to_string()),
        Argument::TemplateLiteral(template) if template.expressions.is_empty() => {
            template.quasis.first().map(|quasi| {
                quasi
                    .value
                    .cooked
                    .as_ref()
                    .unwrap_or(&quasi.value.raw)
                    .to_string()
            })
        }
        _ => None,
    }
}

pub(super) fn collect_calls(argument: &Argument<'_>) -> Vec<CallTarget> {
    let mut collector = CallCollector::default();
    match argument {
        Argument::ArrowFunctionExpression(arrow) => {
            collector.visit_arrow_function_expression(arrow)
        }
        Argument::FunctionExpression(function) => {
            collector.visit_function(function, oxc_syntax::scope::ScopeFlags::empty())
        }
        _ => {}
    }
    collector.calls
}

pub(super) fn collect_function_calls(function: &Function<'_>) -> Vec<CallTarget> {
    let mut collector = CallCollector::default();
    collector.visit_function(function, oxc_syntax::scope::ScopeFlags::empty());
    collector.calls
}

pub(super) fn collect_arrow_calls(function: &ArrowFunctionExpression<'_>) -> Vec<CallTarget> {
    let mut collector = CallCollector::default();
    collector.visit_arrow_function_expression(function);
    collector.calls
}

pub(super) fn integration_annotation_before(source: &str, span: Span) -> Option<String> {
    let prefix = source.get(..span.start as usize)?;
    let trimmed_len = prefix.trim_end().len();
    let trimmed = &prefix[..trimmed_len];
    let end = trimmed.rfind("*/")?;
    if end + 2 != trimmed.len() {
        return None;
    }
    let start = trimmed[..end].rfind("/*")?;
    let body = normalize_block_comment(&trimmed[start + 2..end]);
    let directive = body.trim();
    let rest = directive.strip_prefix("no-mistakes:")?.trim();
    let value = rest
        .strip_prefix("integration=")
        .or_else(|| rest.strip_prefix("integration:"))?
        .trim();
    valid_integration_name(value).then(|| value.to_string())
}

fn normalize_block_comment(body: &str) -> String {
    body.lines()
        .map(|line| line.trim().strip_prefix('*').unwrap_or(line.trim()).trim())
        .collect::<Vec<_>>()
        .join("\n")
}

fn valid_integration_name(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
}
