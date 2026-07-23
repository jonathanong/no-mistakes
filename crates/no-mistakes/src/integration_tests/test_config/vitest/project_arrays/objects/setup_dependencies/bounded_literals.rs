use super::super::super::{shared, Ctx};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{ArrayExpressionElement, Expression};
use std::collections::BTreeSet;

const MAX_DEPTH: usize = 32;
const MAX_SPECIFIERS: usize = 64;
const MAX_PATHS: usize = 256;

pub(super) fn specifiers(expression: &Expression<'_>, ctx: &Ctx<'_, '_>) -> BTreeSet<String> {
    let mut specifiers = Vec::new();
    collect_specifiers(expression, ctx, 0, &mut specifiers);
    specifiers.into_iter().take(MAX_PATHS).collect()
}

fn collect_specifiers(
    expression: &Expression<'_>,
    ctx: &Ctx<'_, '_>,
    depth: usize,
    specifiers: &mut Vec<String>,
) {
    if depth >= MAX_DEPTH || specifiers.len() >= MAX_SPECIFIERS {
        return;
    }
    let expression = shared::expression_value(expression, &ctx.bindings);
    match unwrap_ts_wrappers(expression) {
        Expression::ConditionalExpression(conditional) => {
            collect_specifiers(&conditional.consequent, ctx, depth + 1, specifiers);
            collect_specifiers(&conditional.alternate, ctx, depth + 1, specifiers);
        }
        Expression::ArrayExpression(array) => {
            for element in &array.elements {
                if specifiers.len() >= MAX_SPECIFIERS {
                    break;
                }
                match element {
                    ArrayExpressionElement::Elision(_) => {}
                    ArrayExpressionElement::SpreadElement(spread) => {
                        collect_specifiers(&spread.argument, ctx, depth + 1, specifiers);
                    }
                    _ => collect_specifiers(
                        element
                            .as_expression()
                            .expect("non-spread, non-elision array elements are expressions"),
                        ctx,
                        depth + 1,
                        specifiers,
                    ),
                }
            }
        }
        expression => {
            if let Some(specifier) = shared::optional_string(expression, ctx.source) {
                specifiers.push(specifier);
            }
        }
    }
}
