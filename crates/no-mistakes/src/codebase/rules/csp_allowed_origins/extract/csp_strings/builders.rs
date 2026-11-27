use super::super::super::findings::{CollectedOrigins, DeferredPart, DeferredPolicy};
use super::super::registries::{ConstArrays, RegistryJoin};
use super::detect::{
    array_contains_csp_fragment, csp_header_object_value, expr_contains_csp_fragment,
    is_csp_header_name, is_static_csp_policy_string, object_is_csp_source_map,
};
use super::eval::{csp_array_strings, eval_csp_text, eval_static_string, unwrap_join_array_target};
use super::CspExprResult;
use crate::codebase::ts_source::{line_number, unwrap_ts_wrappers};
use oxc_ast::ast::{Argument, BinaryOperator, CallExpression, Expression, ObjectExpression};
use oxc_span::GetSpan;
use std::borrow::Cow;
use std::collections::HashSet;

pub(in super::super) fn try_collect_call(
    call: &CallExpression<'_>,
    source: &str,
    out: &mut CollectedOrigins,
    arrays: &ConstArrays,
) -> CspExprResult {
    if let Some(join_result) = try_eval_join_call(call, arrays) {
        return match join_result {
            JoinEval::Resolved(joined) => {
                if is_static_csp_policy_string(&joined) {
                    out.csp_strings.push(joined);
                }
                CspExprResult::Resolved
            }
            JoinEval::Deferred(parts) => {
                out.deferred_policies.push(DeferredPolicy {
                    line: line_number(source, call.span.start),
                    parts,
                });
                CspExprResult::Resolved
            }
            JoinEval::Unanalyzable => CspExprResult::Unanalyzable(call.span.start),
            JoinEval::Ignored => CspExprResult::Ignored,
        };
    }
    if let Some(span) = header_setter_unanalyzable_span(call, arrays) {
        return CspExprResult::Unanalyzable(span);
    }
    if call
        .arguments
        .iter()
        .any(|arg| arg_is_unresolvable_csp(arg, arrays))
    {
        CspExprResult::Unanalyzable(call.span.start)
    } else {
        CspExprResult::Ignored
    }
}

pub(in super::super) fn try_collect_object(
    obj: &ObjectExpression<'_>,
    arrays: &ConstArrays,
) -> CspExprResult {
    if object_is_csp_source_map(obj) {
        return CspExprResult::Unanalyzable(obj.span.start);
    }
    if let Some(value) = csp_header_object_value(obj) {
        if !is_resolvable_csp_expr(value, arrays) {
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
        | Expression::UnaryExpression(_) => false,
        Expression::TemplateLiteral(tpl) if tpl.expressions.is_empty() => false,
        Expression::BinaryExpression(bin) if bin.operator != BinaryOperator::Addition => false,
        Expression::Identifier(ident) => !resolved_bindings.contains(ident.name.as_str()),
        Expression::CallExpression(call)
            if call.arguments.is_empty()
                && matches!(
                    unwrap_ts_wrappers(&call.callee),
                    Expression::Identifier(id) if resolved_bindings.contains(id.name.as_str())
                ) =>
        {
            false
        }
        _ => true,
    }
}

enum JoinEval {
    Resolved(String),
    Deferred(Vec<DeferredPart>),
    Unanalyzable,
    Ignored,
}

fn try_eval_join_call(call: &CallExpression<'_>, arrays: &ConstArrays) -> Option<JoinEval> {
    if let Some(joined) = arrays.eval_join(call) {
        return Some(match joined {
            RegistryJoin::Resolved(text) => JoinEval::Resolved(text),
            RegistryJoin::Deferred(parts) => JoinEval::Deferred(parts),
        });
    }
    let Expression::StaticMemberExpression(member) = unwrap_ts_wrappers(&call.callee) else {
        return None;
    };
    if member.optional || member.property.name.as_str() != "join" || call.arguments.len() > 1 {
        return None;
    }
    let (array, unfiltered) = unwrap_join_array_target(&member.object)?;
    let separator = match call.arguments.first() {
        None => Some(Cow::Borrowed(",")),
        Some(arg) => arg.as_expression().and_then(eval_static_string),
    };
    if unfiltered {
        if let (Some(sep), Some(items)) = (separator, csp_array_strings(array, arrays)) {
            return Some(JoinEval::Resolved(items.join(&sep)));
        }
    }
    if array_contains_csp_fragment(array) {
        Some(JoinEval::Unanalyzable)
    } else {
        Some(JoinEval::Ignored)
    }
}

fn header_setter_unanalyzable_span(call: &CallExpression<'_>, arrays: &ConstArrays) -> Option<u32> {
    if call.arguments.len() < 2 {
        return None;
    }
    let first = call.arguments[0].as_expression()?;
    if !eval_static_string(first).is_some_and(|name| is_csp_header_name(&name)) {
        return None;
    }
    let second = &call.arguments[1];
    match second.as_expression() {
        Some(value) if is_resolvable_csp_expr(value, arrays) => None,
        _ => Some(second.span().start),
    }
}

fn arg_is_unresolvable_csp(arg: &Argument<'_>, arrays: &ConstArrays) -> bool {
    match arg {
        Argument::SpreadElement(spread) => expr_contains_csp_fragment(&spread.argument),
        other => {
            let expr = other.to_expression();
            expr_contains_csp_fragment(expr) && !is_resolvable_csp_expr(expr, arrays)
        }
    }
}

fn is_resolvable_csp_expr(expr: &Expression<'_>, arrays: &ConstArrays) -> bool {
    let inner = unwrap_ts_wrappers(expr);
    if let Some(text) = eval_csp_text(inner, arrays) {
        return is_static_csp_policy_string(&text);
    }
    if let Expression::TemplateLiteral(template) = inner {
        if arrays.defer_template(template).is_some() {
            return true;
        }
    }
    match inner {
        Expression::ArrayExpression(array) => csp_array_strings(array, arrays)
            .is_some_and(|items| items.iter().any(|item| is_static_csp_policy_string(item))),
        Expression::CallExpression(call) => match try_eval_join_call(call, arrays) {
            Some(JoinEval::Resolved(joined)) => is_static_csp_policy_string(&joined),
            Some(JoinEval::Deferred(_)) => true,
            _ => false,
        },
        _ => false,
    }
}
