use super::super::super::{shared, Ctx};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{ArrayExpressionElement, Expression};
use std::collections::BTreeSet;
use std::path::PathBuf;

const MAX_DEPTH: usize = 32;
const MAX_SPECIFIERS: usize = 64;
const MAX_PATHS: usize = 256;

pub(super) fn trigger_paths(expression: &Expression<'_>, ctx: &Ctx<'_, '_>) -> BTreeSet<PathBuf> {
    let mut specifiers = Vec::new();
    collect_specifiers(expression, ctx, 0, &mut specifiers);
    let mut paths = BTreeSet::new();
    for specifier in specifiers {
        if let Some(path) = ctx.resolver.resolve(&specifier, ctx.path) {
            paths.insert(path);
        }
        for candidate in ctx.resolver.resolution_candidates(&specifier, ctx.path) {
            if paths.len() >= MAX_PATHS {
                return paths;
            }
            paths.insert(candidate);
        }
    }
    paths
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
                    _ => {
                        if let Some(expression) = element.as_expression() {
                            collect_specifiers(expression, ctx, depth + 1, specifiers);
                        }
                    }
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
