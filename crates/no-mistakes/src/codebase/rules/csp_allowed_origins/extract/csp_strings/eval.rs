use super::super::registries::{ConstArrays, RegistryJoin};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{
    ArrayExpression, ArrayExpressionElement, BinaryExpression, BinaryOperator, Expression,
    TemplateLiteral,
};
use std::borrow::Cow;

pub(in super::super) fn static_template_text<'a>(
    template: &'a TemplateLiteral<'a>,
) -> Option<Cow<'a, str>> {
    if !template.expressions.is_empty() {
        return None;
    }
    let quasi = template.quasis.first()?;
    quasi.value.cooked?;
    Some(crate::ast::template_quasi_text(quasi))
}

pub(in super::super) fn eval_static_string<'a>(
    expression: &'a Expression<'a>,
) -> Option<Cow<'a, str>> {
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

pub(in super::super) fn eval_csp_text<'a>(
    expression: &'a Expression<'a>,
    arrays: &ConstArrays,
) -> Option<Cow<'a, str>> {
    match unwrap_ts_wrappers(expression) {
        Expression::StringLiteral(value) => Some(crate::ast::js_literal(value)),
        Expression::TemplateLiteral(template) => eval_csp_template(template, arrays),
        Expression::BinaryExpression(bin) if bin.operator == BinaryOperator::Addition => {
            eval_csp_binary(bin, arrays).map(Cow::Owned)
        }
        Expression::CallExpression(call) => match arrays.eval_join(call) {
            Some(RegistryJoin::Resolved(text)) => Some(Cow::Owned(text)),
            _ => None,
        },
        _ => None,
    }
}

pub(super) fn eval_csp_template<'a>(
    template: &'a TemplateLiteral<'a>,
    arrays: &ConstArrays,
) -> Option<Cow<'a, str>> {
    if template.expressions.is_empty() {
        return static_template_text(template);
    }
    let mut out = String::new();
    for (idx, quasi) in template.quasis.iter().enumerate() {
        quasi.value.cooked?;
        out.push_str(&crate::ast::template_quasi_text(quasi));
        let Some(expr) = template.expressions.get(idx) else {
            break;
        };
        if let Some(static_part) = eval_csp_text(expr, arrays) {
            out.push_str(&static_part);
            continue;
        }
        let next_quasi = template.quasis.get(idx + 1)?;
        next_quasi.value.cooked?;
        let next_text = crate::ast::template_quasi_text(next_quasi);
        if is_csp_nonce_or_hash_slot(&out, &next_text) {
            out.push_str("placeholder");
        } else {
            return None;
        }
    }
    Some(Cow::Owned(out))
}

pub(super) fn eval_csp_binary(expr: &BinaryExpression<'_>, arrays: &ConstArrays) -> Option<String> {
    let mut pieces = Vec::new();
    collect_add_pieces(&expr.left, &mut pieces, arrays);
    collect_add_pieces(&expr.right, &mut pieces, arrays);
    let mut out = String::new();
    for (idx, piece) in pieces.iter().enumerate() {
        match piece {
            Some(text) => out.push_str(text),
            None => {
                let Some(Some(next)) = pieces.get(idx + 1) else {
                    return None;
                };
                if is_csp_nonce_or_hash_slot(&out, next) {
                    out.push_str("placeholder");
                } else {
                    return None;
                }
            }
        }
    }
    Some(out)
}

fn collect_add_pieces(
    expr: &Expression<'_>,
    pieces: &mut Vec<Option<String>>,
    arrays: &ConstArrays,
) {
    match unwrap_ts_wrappers(expr) {
        Expression::BinaryExpression(bin) if bin.operator == BinaryOperator::Addition => {
            collect_add_pieces(&bin.left, pieces, arrays);
            collect_add_pieces(&bin.right, pieces, arrays);
        }
        other => match eval_csp_text(other, arrays) {
            Some(text) => {
                if let Some(Some(prev)) = pieces.last_mut() {
                    prev.push_str(&text);
                } else {
                    pieces.push(Some(text.into_owned()));
                }
            }
            None => pieces.push(None),
        },
    }
}

fn is_csp_nonce_or_hash_slot(before: &str, after: &str) -> bool {
    let Some(rest_after) = after.strip_prefix('\'') else {
        return false;
    };
    if !rest_after.is_empty()
        && !rest_after.starts_with(|ch: char| ch.is_whitespace() || matches!(ch, ';' | ','))
    {
        return false;
    }
    let lower = before.to_ascii_lowercase();
    ["'nonce-", "'sha256-", "'sha384-", "'sha512-"]
        .iter()
        .any(|prefix| {
            lower.strip_suffix(prefix).is_some_and(|head| {
                head.is_empty()
                    || head.ends_with(|ch: char| ch.is_whitespace() || matches!(ch, ';' | ','))
            })
        })
}

pub(super) fn csp_array_strings(
    array: &ArrayExpression<'_>,
    arrays: &ConstArrays,
) -> Option<Vec<String>> {
    let mut items = Vec::with_capacity(array.elements.len());
    for element in &array.elements {
        let ArrayExpressionElement::SpreadElement(_) = element else {
            let expr = element.as_expression()?;
            items.push(eval_csp_text(expr, arrays)?.into_owned());
            continue;
        };
        return None;
    }
    Some(items)
}

pub(super) fn unwrap_join_array_target<'a>(
    expr: &'a Expression<'a>,
) -> Option<(&'a ArrayExpression<'a>, bool)> {
    match unwrap_ts_wrappers(expr) {
        Expression::ArrayExpression(array) => Some((array, true)),
        Expression::CallExpression(call) => {
            let Expression::StaticMemberExpression(member) = unwrap_ts_wrappers(&call.callee)
            else {
                return None;
            };
            if member.optional || member.property.name.as_str() != "filter" {
                return None;
            }
            match unwrap_ts_wrappers(&member.object) {
                Expression::ArrayExpression(array) => Some((array, false)),
                _ => None,
            }
        }
        _ => None,
    }
}
