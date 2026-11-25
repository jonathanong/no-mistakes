use super::detect::{
    array_contains_csp_fragment, csp_header_object_value, expr_contains_csp_fragment,
    is_csp_header_name, is_static_csp_policy_string, object_is_csp_source_map,
};
use super::eval::{
    csp_array_strings, eval_csp_text, eval_static_string, unwrap_join_array_target,
};
use super::CspExprResult;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{
    Argument, BinaryOperator, CallExpression, Expression, ObjectExpression,
};
use oxc_span::GetSpan;
use std::borrow::Cow;
use std::collections::HashSet;

pub(in super::super) fn try_collect_call(
    call: &CallExpression<'_>,
    out: &mut Vec<String>,
) -> CspExprResult {
    if let Expression::StaticMemberExpression(member) = unwrap_ts_wrappers(&call.callee) {
        if !member.optional && member.property.name.as_str() == "join" && call.arguments.len() <= 1
        {
            if let Some((array, unfiltered)) = unwrap_join_array_target(&member.object) {
                let separator = match call.arguments.first() {
                    None => Some(Cow::Borrowed(",")),
                    Some(arg) => arg.as_expression().and_then(eval_static_string),
                };
                if unfiltered {
                    if let (Some(sep), Some(items)) = (separator, csp_array_strings(array)) {
                        let joined = items.join(&sep);
                        if is_static_csp_policy_string(&joined) {
                            out.push(joined);
                        }
                        return CspExprResult::Resolved;
                    }
                }
                if array_contains_csp_fragment(array) {
                    return CspExprResult::Unanalyzable(call.span.start);
                }
                return CspExprResult::Ignored;
            }
        }
    }
    if let Some(span) = header_setter_unanalyzable_span(call) {
        return CspExprResult::Unanalyzable(span);
    }
    if call.arguments.iter().any(arg_is_unresolvable_csp) {
        CspExprResult::Unanalyzable(call.span.start)
    } else {
        CspExprResult::Ignored
    }
}

pub(in super::super) fn try_collect_object(obj: &ObjectExpression<'_>) -> CspExprResult {
    if object_is_csp_source_map(obj) {
        return CspExprResult::Unanalyzable(obj.span.start);
    }
    if let Some(value) = csp_header_object_value(obj) {
        if !is_resolvable_csp_expr(value) {
            return CspExprResult::Unanalyzable(value.span().start);
        }
    }
    CspExprResult::Ignored
}

pub(in super::super) fn is_dynamic_policy_candidate(
    expr: &Expression<'_>,
    resolved_bindings: &HashSet<String>,
) -> bool {
    match unwrap_ts_wrappers(expr) {
        Expression::StringLiteral(_)
        | Expression::BooleanLiteral(_)
        | Expression::NumericLiteral(_)
        | Expression::NullLiteral(_)
        | Expression::BigIntLiteral(_)
        | Expression::RegExpLiteral(_)
        | Expression::UnaryExpression(_)
        | Expression::FunctionExpression(_)
        | Expression::ArrowFunctionExpression(_) => false,
        Expression::TemplateLiteral(tpl) if tpl.expressions.is_empty() => false,
        Expression::BinaryExpression(bin) if bin.operator != BinaryOperator::Addition => false,
        Expression::Identifier(ident) => {
            ident.name.as_str() != "undefined" && !resolved_bindings.contains(ident.name.as_str())
        }
        Expression::CallExpression(call) => {
            if call.arguments.is_empty() {
                if let Expression::Identifier(ident) = unwrap_ts_wrappers(&call.callee) {
                    if resolved_bindings.contains(ident.name.as_str()) {
                        return false;
                    }
                }
            }
            true
        }
        _ => true,
    }
}

fn header_setter_unanalyzable_span(call: &CallExpression<'_>) -> Option<u32> {
    if call.arguments.len() < 2 {
        return None;
    }
    let first = call.arguments[0].as_expression()?;
    if !eval_static_string(first).is_some_and(|name| is_csp_header_name(&name)) {
        return None;
    }
    let value = call.arguments[1].as_expression()?;
    (!is_resolvable_csp_expr(value)).then(|| value.span().start)
}

fn arg_is_unresolvable_csp(arg: &Argument<'_>) -> bool {
    match arg {
        Argument::SpreadElement(spread) => expr_contains_csp_fragment(&spread.argument),
        other => other
            .as_expression()
            .is_some_and(|expr| expr_contains_csp_fragment(expr) && !is_resolvable_csp_expr(expr)),
    }
}

fn is_resolvable_csp_expr(expr: &Expression<'_>) -> bool {
    let inner = unwrap_ts_wrappers(expr);
    if let Some(text) = eval_csp_text(inner) {
        return is_static_csp_policy_string(&text);
    }
    match inner {
        Expression::ArrayExpression(array) => csp_array_strings(array)
            .is_some_and(|items| items.iter().any(|item| is_static_csp_policy_string(item))),
        Expression::CallExpression(call) => {
            let Expression::StaticMemberExpression(member) = unwrap_ts_wrappers(&call.callee)
            else {
                return false;
            };
            if member.optional || member.property.name.as_str() != "join" || call.arguments.len() > 1
            {
                return false;
            }
            let Some((array, true)) = unwrap_join_array_target(&member.object) else {
                return false;
            };
            let sep = match call.arguments.first() {
                None => Some(Cow::Borrowed(",")),
                Some(arg) => arg.as_expression().and_then(eval_static_string),
            };
            matches!(
                (sep, csp_array_strings(array)),
                (Some(s), Some(items)) if is_static_csp_policy_string(&items.join(&s))
            )
        }
        _ => false,
    }
}
