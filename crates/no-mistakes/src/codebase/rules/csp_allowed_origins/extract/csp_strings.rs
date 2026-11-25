use super::super::CspDirective;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{
    ArrayExpression, ArrayExpressionElement, BinaryExpression, BinaryOperator, CallExpression,
    Expression, TemplateLiteral,
};
use std::borrow::Cow;
use std::collections::BTreeSet;

pub(super) fn try_collect_join_call(call: &CallExpression<'_>, out: &mut Vec<String>) -> bool {
    let Expression::StaticMemberExpression(member) = unwrap_ts_wrappers(&call.callee) else {
        return false;
    };
    if member.optional || member.property.name.as_str() != "join" || call.arguments.len() > 1 {
        return false;
    }
    let Expression::ArrayExpression(array) = unwrap_ts_wrappers(&member.object) else {
        return false;
    };
    let separator = match call.arguments.first() {
        None => Cow::Borrowed(","),
        Some(arg) => {
            let Some(sep) = arg.as_expression().and_then(eval_static_string) else {
                return false;
            };
            sep
        }
    };
    let Some(items) = static_array_strings(array) else {
        return false;
    };
    let joined = items.join(&separator);
    if joined.contains("-src") {
        out.push(joined);
    }
    true
}

pub(super) fn try_collect_array(array: &ArrayExpression<'_>, out: &mut Vec<String>) -> bool {
    let Some(items) = static_array_strings(array) else {
        return false;
    };
    if !items.iter().any(|item| item.contains("-src")) {
        return true;
    }
    if has_overlapping_directives(&items) {
        for item in items {
            if item.contains("-src") {
                out.push(item);
            }
        }
    } else {
        out.push(items.join("; "));
    }
    true
}

pub(super) fn try_collect_binary(expr: &BinaryExpression<'_>, out: &mut Vec<String>) -> bool {
    if expr.operator != BinaryOperator::Addition {
        return false;
    }
    let Some(left) = eval_static_string(&expr.left) else {
        return false;
    };
    let Some(right) = eval_static_string(&expr.right) else {
        return false;
    };
    let combined = format!("{left}{right}");
    if combined.contains("-src") {
        out.push(combined);
    }
    true
}

pub(super) fn static_template_text<'a>(template: &'a TemplateLiteral<'a>) -> Option<Cow<'a, str>> {
    if !template.expressions.is_empty() {
        return None;
    }
    let quasi = template.quasis.first()?;
    quasi.value.cooked?;
    Some(crate::ast::template_quasi_text(quasi))
}

pub(super) fn eval_static_string<'a>(expression: &'a Expression<'a>) -> Option<Cow<'a, str>> {
    match unwrap_ts_wrappers(expression) {
        Expression::StringLiteral(value) => Some(crate::ast::js_literal(value)),
        Expression::TemplateLiteral(template) => static_template_text(template),
        Expression::BinaryExpression(bin) if bin.operator == BinaryOperator::Addition => {
            let left = eval_static_string(&bin.left)?;
            let right = eval_static_string(&bin.right)?;
            Some(Cow::Owned(format!("{left}{right}")))
        }
        _ => None,
    }
}

fn static_array_strings(array: &ArrayExpression<'_>) -> Option<Vec<String>> {
    let mut items = Vec::with_capacity(array.elements.len());
    for element in &array.elements {
        let ArrayExpressionElement::SpreadElement(_) = element else {
            let expr = element.as_expression()?;
            items.push(eval_static_string(expr)?.into_owned());
            continue;
        };
        return None;
    }
    Some(items)
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
