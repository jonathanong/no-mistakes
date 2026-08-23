use crate::codebase::ts_source::byte_offset_to_line;
use oxc_ast::ast::{Argument, CallExpression, Expression};
use oxc_span::GetSpan;

/// A direct identifier or one-level static member call recorded during the
/// shared TypeScript fact pass. Query consumers select the relevant callee
/// names without reparsing callers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallSiteFact {
    pub callee: String,
    /// Whether invocation is conditional through an optional call or member chain.
    pub is_optional: bool,
    pub line: u32,
    pub caller: Option<String>,
    pub static_arg_source: Option<String>,
    pub arg_count: usize,
    pub has_spread: bool,
    pub args: Vec<&'static str>,
}

fn callee_name(callee: &Expression<'_>) -> Option<String> {
    match callee {
        Expression::Identifier(identifier) => Some(identifier.name.to_string()),
        Expression::StaticMemberExpression(member) => match &member.object {
            Expression::Identifier(object) => Some(format!(
                "{}.{}",
                object.name.as_str(),
                member.property.name.as_str()
            )),
            Expression::ThisExpression(_) => Some(format!("this.{}", member.property.name)),
            _ => None,
        },
        _ => None,
    }
}

fn static_first_string_arg_source(call: &CallExpression<'_>, source: &str) -> Option<String> {
    let argument = call.arguments.first()?;
    static_string_arg_source(argument, source)
}

fn static_string_arg_source(argument: &Argument<'_>, source: &str) -> Option<String> {
    match argument {
        Argument::ParenthesizedExpression(parenthesized) => {
            static_string_expression_source(&parenthesized.expression, source)
        }
        Argument::StringLiteral(_) => Some(crate::ast::span_text(source, argument.span()).into()),
        Argument::TemplateLiteral(template) if template.expressions.is_empty() => {
            Some(crate::ast::span_text(source, argument.span()).into())
        }
        _ => None,
    }
}

fn static_string_expression_source(expression: &Expression<'_>, source: &str) -> Option<String> {
    match expression {
        Expression::ParenthesizedExpression(parenthesized) => {
            static_string_expression_source(&parenthesized.expression, source)
        }
        Expression::StringLiteral(_) => {
            Some(crate::ast::span_text(source, expression.span()).into())
        }
        Expression::TemplateLiteral(template) if template.expressions.is_empty() => {
            Some(crate::ast::span_text(source, expression.span()).into())
        }
        _ => None,
    }
}

/// Coarse syntactic shape of one argument — no type inference.
fn arg_tag(arg: &Argument<'_>) -> &'static str {
    match arg {
        Argument::SpreadElement(_) => "spread",
        Argument::StringLiteral(_) | Argument::TemplateLiteral(_) => "string",
        Argument::NumericLiteral(_) | Argument::BigIntLiteral(_) => "number",
        Argument::BooleanLiteral(_) => "boolean",
        Argument::NullLiteral(_) => "null",
        Argument::Identifier(_) => "identifier",
        Argument::ObjectExpression(_) => "object",
        Argument::ArrayExpression(_) => "array",
        Argument::ArrowFunctionExpression(_) | Argument::FunctionExpression(_) => "arrow",
        Argument::CallExpression(_) => "call",
        _ => "other",
    }
}

pub(crate) fn record_call_site(
    source: &str,
    caller: Option<&str>,
    call: &CallExpression<'_>,
    sites: &mut Vec<CallSiteFact>,
) {
    let Some(callee) = callee_name(&call.callee) else {
        return;
    };
    let is_optional = call.optional
        || matches!(
            &call.callee,
            Expression::StaticMemberExpression(member) if member.optional
        );
    sites.push(CallSiteFact {
        callee,
        is_optional,
        line: byte_offset_to_line(source, call.span.start as usize),
        caller: caller.map(str::to_string),
        static_arg_source: static_first_string_arg_source(call, source),
        arg_count: call.arguments.len(),
        has_spread: call
            .arguments
            .iter()
            .any(|arg| matches!(arg, Argument::SpreadElement(_))),
        args: call.arguments.iter().map(arg_tag).collect(),
    });
}
