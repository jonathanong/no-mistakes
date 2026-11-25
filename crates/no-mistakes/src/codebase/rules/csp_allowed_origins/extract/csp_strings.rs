use super::super::CspDirective;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{
    ArrayExpression, BinaryExpression, BinaryOperator, CallExpression, ConditionalExpression,
    Expression, LogicalExpression, TemplateLiteral,
};
use std::borrow::Cow;
use std::collections::BTreeSet;

mod detect;
mod eval;

use detect::{
    array_contains_csp_fragment, expr_contains_csp_fragment, template_contains_csp_fragment,
};
pub(super) use eval::{eval_static_string, static_template_text};
use eval::{csp_array_strings, eval_csp_binary, eval_csp_template, unwrap_join_array_target};

pub(super) enum CspExprResult {
    Resolved,
    Unanalyzable(u32),
    Ignored,
}

pub(super) fn try_collect_join_call(
    call: &CallExpression<'_>,
    out: &mut Vec<String>,
) -> CspExprResult {
    let Expression::StaticMemberExpression(member) = unwrap_ts_wrappers(&call.callee) else {
        return CspExprResult::Ignored;
    };
    if member.optional || member.property.name.as_str() != "join" || call.arguments.len() > 1 {
        return CspExprResult::Ignored;
    }
    let Some((array, unfiltered)) = unwrap_join_array_target(&member.object) else {
        return CspExprResult::Ignored;
    };
    let separator = match call.arguments.first() {
        None => Some(Cow::Borrowed(",")),
        Some(arg) => arg.as_expression().and_then(eval_static_string),
    };
    if unfiltered {
        if let (Some(sep), Some(items)) = (separator, csp_array_strings(array)) {
            let joined = items.join(&sep);
            if joined.contains("-src") {
                out.push(joined);
            }
            return CspExprResult::Resolved;
        }
    }
    if array_contains_csp_fragment(array) {
        CspExprResult::Unanalyzable(call.span.start)
    } else {
        CspExprResult::Ignored
    }
}

pub(super) fn try_collect_array(
    array: &ArrayExpression<'_>,
    out: &mut Vec<String>,
) -> CspExprResult {
    if let Some(items) = csp_array_strings(array) {
        if items.iter().any(|item| item.contains("-src")) {
            if has_overlapping_directives(&items) {
                for item in items {
                    if item.contains("-src") {
                        out.push(item);
                    }
                }
            } else {
                out.push(items.join("; "));
            }
        }
        return CspExprResult::Resolved;
    }
    if array_contains_csp_fragment(array) {
        CspExprResult::Unanalyzable(array.span.start)
    } else {
        CspExprResult::Ignored
    }
}

pub(super) fn try_collect_binary(
    expr: &BinaryExpression<'_>,
    out: &mut Vec<String>,
) -> CspExprResult {
    if expr.operator != BinaryOperator::Addition {
        return CspExprResult::Ignored;
    }
    if let Some(combined) = eval_csp_binary(expr) {
        if combined.contains("-src") {
            out.push(combined);
        }
        return CspExprResult::Resolved;
    }
    if expr_contains_csp_fragment(&expr.left) || expr_contains_csp_fragment(&expr.right) {
        CspExprResult::Unanalyzable(expr.span.start)
    } else {
        CspExprResult::Ignored
    }
}

pub(super) fn try_collect_template(
    template: &TemplateLiteral<'_>,
    out: &mut Vec<String>,
) -> CspExprResult {
    if let Some(value) = eval_csp_template(template) {
        if value.contains("-src") {
            out.push(value.into_owned());
        }
        return CspExprResult::Resolved;
    }
    if template_contains_csp_fragment(template) {
        CspExprResult::Unanalyzable(template.span.start)
    } else {
        CspExprResult::Ignored
    }
}

pub(super) fn try_collect_conditional(cond: &ConditionalExpression<'_>) -> CspExprResult {
    if expr_contains_csp_fragment(&cond.consequent) || expr_contains_csp_fragment(&cond.alternate) {
        CspExprResult::Unanalyzable(cond.span.start)
    } else {
        CspExprResult::Ignored
    }
}

pub(super) fn try_collect_logical(logical: &LogicalExpression<'_>) -> CspExprResult {
    if expr_contains_csp_fragment(&logical.left) || expr_contains_csp_fragment(&logical.right) {
        CspExprResult::Unanalyzable(logical.span.start)
    } else {
        CspExprResult::Ignored
    }
}

fn has_overlapping_directives(items: &[String]) -> bool {
    let mut seen = BTreeSet::new();
    for item in items {
        let mut item_dirs = BTreeSet::new();
        for part in item.split(';') {
            if let Some(dir) = part.split_whitespace().next().and_then(CspDirective::parse) {
                item_dirs.insert(dir);
            }
        }
        for dir in item_dirs {
            if !seen.insert(dir) {
                return true;
            }
        }
    }
    false
}
