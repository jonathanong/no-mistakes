use super::{is_sql_tag, SqlTagNames};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{ArrayExpressionElement, Expression, TemplateLiteral};
mod collect;
pub(super) use collect::collect_fragment_bindings;

/// Whether a trusted SQL tag's template interpolates another SQL fragment
/// rather than a parameterized value. Such a template is spliced from
/// several pieces at runtime, so its quasi text alone is not the statement
/// that executes: the fragment's own SQL (or one of several branch
/// fragments) replaces the `sql_placeholder_N` that would otherwise stand for
/// a bind value.
pub(super) fn interpolates_sql_fragment(
    template: &TemplateLiteral<'_>,
    is_shadowed: &mut impl FnMut(&str) -> bool,
    tags: &SqlTagNames,
) -> bool {
    template
        .expressions
        .iter()
        .any(|expr| is_sql_fragment(expr, is_shadowed, tags))
}

/// A trusted tagged template, a call on the trusted tag (`sql(...)`,
/// `sql.raw(...)`, `sql.join(...)`), a member call on another fragment
/// (`sql`…`.append(...)`), a binding known to hold one, or a conditional,
/// logical, sequence, or array expression that can evaluate to one.
pub(in crate::codebase::postgres::embedded) fn is_sql_fragment(
    expr: &Expression<'_>,
    is_shadowed: &mut impl FnMut(&str) -> bool,
    tags: &SqlTagNames,
) -> bool {
    yields_fragment(expr, is_shadowed, tags, &mut |name, called| {
        if called {
            tags.fragment_functions.contains(name)
        } else {
            tags.fragments.contains(name)
        }
    })
}

fn yields_fragment(
    expr: &Expression<'_>,
    is_shadowed: &mut impl FnMut(&str) -> bool,
    tags: &SqlTagNames,
    bound_fragment: &mut impl FnMut(&str, bool) -> bool,
) -> bool {
    match unwrap_ts_wrappers(expr) {
        Expression::TaggedTemplateExpression(tagged) => is_sql_tag(&tagged.tag, is_shadowed, tags),
        Expression::CallExpression(call) => match unwrap_ts_wrappers(&call.callee) {
            Expression::StaticMemberExpression(member) => {
                is_sql_tag(&member.object, is_shadowed, tags)
                    || yields_fragment(&member.object, is_shadowed, tags, bound_fragment)
            }
            Expression::ComputedMemberExpression(member) => {
                is_sql_tag(&member.object, is_shadowed, tags)
                    || yields_fragment(&member.object, is_shadowed, tags, bound_fragment)
            }
            Expression::Identifier(ident) => {
                is_sql_tag(&call.callee, is_shadowed, tags)
                    || bound_fragment(ident.name.as_str(), true)
            }
            callee => is_sql_tag(callee, is_shadowed, tags),
        },
        Expression::ConditionalExpression(conditional) => {
            yields_fragment(&conditional.consequent, is_shadowed, tags, bound_fragment)
                || yields_fragment(&conditional.alternate, is_shadowed, tags, bound_fragment)
        }
        Expression::LogicalExpression(logical) => {
            yields_fragment(&logical.left, is_shadowed, tags, bound_fragment)
                || yields_fragment(&logical.right, is_shadowed, tags, bound_fragment)
        }
        Expression::SequenceExpression(sequence) => sequence
            .expressions
            .last()
            .is_some_and(|last| yields_fragment(last, is_shadowed, tags, bound_fragment)),
        Expression::ArrayExpression(array) => array.elements.iter().any(|element| match element {
            ArrayExpressionElement::SpreadElement(spread) => {
                yields_fragment(&spread.argument, is_shadowed, tags, bound_fragment)
            }
            element => element
                .as_expression()
                .is_some_and(|expr| yields_fragment(expr, is_shadowed, tags, bound_fragment)),
        }),
        Expression::Identifier(ident) => bound_fragment(ident.name.as_str(), false),
        _ => false,
    }
}
