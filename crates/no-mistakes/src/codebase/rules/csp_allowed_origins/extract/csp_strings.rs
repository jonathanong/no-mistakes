use super::super::findings::{CollectedOrigins, DeferredPolicy};
use super::super::CspDirective;
use super::registries::ConstArrays;
use crate::codebase::ts_source::line_number;
use oxc_ast::ast::{
    ArrayExpression, BinaryExpression, BinaryOperator, ConditionalExpression, LogicalExpression,
    TemplateLiteral,
};
use std::collections::BTreeSet;

mod builders;
mod detect;
mod eval;

pub(super) use builders::{is_dynamic_policy_candidate, try_collect_call, try_collect_object};
use detect::{
    array_contains_csp_fragment, expr_contains_csp_fragment, template_contains_csp_fragment,
};
pub(super) use detect::{is_csp_identifier, is_static_csp_policy_string};
use eval::{csp_array_strings, eval_csp_binary, eval_csp_template};
pub(super) use eval::{eval_csp_text, eval_static_string, static_template_text};

pub(super) enum CspExprResult {
    Resolved,
    Unanalyzable(u32),
    Ignored,
}

pub(super) fn try_collect_array(
    array: &ArrayExpression<'_>,
    out: &mut CollectedOrigins,
    arrays: &ConstArrays,
) -> CspExprResult {
    if let Some(items) = csp_array_strings(array, arrays) {
        if items.iter().any(|item| is_static_csp_policy_string(item)) {
            if has_overlapping_directives(&items) {
                for item in items {
                    if is_static_csp_policy_string(&item) {
                        out.csp_strings.push(item);
                    }
                }
            } else {
                out.csp_strings.push(items.join("; "));
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
    out: &mut CollectedOrigins,
    arrays: &ConstArrays,
) -> CspExprResult {
    if expr.operator != BinaryOperator::Addition {
        return CspExprResult::Ignored;
    }
    if let Some(combined) = eval_csp_binary(expr, arrays) {
        if is_static_csp_policy_string(&combined) {
            out.csp_strings.push(combined);
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
    source: &str,
    out: &mut CollectedOrigins,
    arrays: &ConstArrays,
) -> CspExprResult {
    if let Some(value) = eval_csp_template(template, arrays) {
        if is_static_csp_policy_string(&value) {
            out.csp_strings.push(value.into_owned());
        }
        return CspExprResult::Resolved;
    }
    if let Some(parts) = arrays.defer_template(template) {
        out.deferred_policies.push(DeferredPolicy {
            line: line_number(source, template.span.start),
            parts,
        });
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
