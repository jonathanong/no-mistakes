use super::names::{Names, ValueKind};
use crate::fx::{FxHashMap, FxHashSet};
use oxc_ast::ast::*;
use oxc_ast_visit::{walk, Visit};
#[path = "array_callback_aliases.rs"]
mod callback_aliases;
pub(super) fn literal_length(array: &ArrayExpression<'_>) -> Option<usize> {
    (!array
        .elements
        .iter()
        .any(|element| matches!(element, ArrayExpressionElement::SpreadElement(_))))
    .then_some(array.elements.len())
}
pub(super) fn preserve_empty_array(
    kind: Option<ValueKind>,
    init: Option<&Expression<'_>>,
    declaration_kind: VariableDeclarationKind,
) -> Option<ValueKind> {
    let Some(ValueKind::Array { element, .. }) = kind else {
        return kind;
    };
    let empty = declaration_kind == VariableDeclarationKind::Const
        && matches!(init.map(Expression::without_parentheses), Some(Expression::ArrayExpression(array))
            if array.elements.iter().all(|item| matches!(item, ArrayExpressionElement::Elision(_))));
    let length = (declaration_kind == VariableDeclarationKind::Const)
        .then(|| init.map(Expression::without_parentheses))
        .flatten()
        .and_then(|init| match init {
            Expression::ArrayExpression(array) => literal_length(array),
            _ => None,
        });
    Some(ValueKind::Array {
        element,
        empty,
        length,
    })
}

pub(super) fn invalidate(names: &mut Names, program: &Program<'_>) {
    #[cfg(feature = "test-instrumentation")]
    callback_aliases::reset_summaries();
    let candidates: FxHashSet<_> = names
        .map
        .iter()
        .filter(|(_, values)| {
            !values.is_empty()
                && values.iter().all(|value| {
                    matches!(
                        value,
                        Some(ValueKind::Array {
                            length: Some(_),
                            ..
                        })
                    )
                })
        })
        .map(|(name, _)| name.clone())
        .collect();
    if candidates.is_empty() {
        return;
    }
    let mut uses = ArrayUses {
        candidates,
        global_array: names.global("Array"),
        ..ArrayUses::default()
    };
    uses.visit_program(program);
    callback_aliases::collect(names, program, &mut uses);
    names.unstable_array_lengths = uses.instability();
}

pub(super) fn source_array<'e>(expression: &'e Expression<'_>) -> Option<&'e str> {
    match expression.without_parentheses() {
        Expression::Identifier(id) => Some(id.name.as_str()),
        Expression::TSAsExpression(cast) => source_array(&cast.expression),
        Expression::TSNonNullExpression(inner) => source_array(&inner.expression),
        Expression::CallExpression(call) => {
            let Expression::StaticMemberExpression(member) = call.callee.without_parentheses()
            else {
                return None;
            };
            if member.property.name.as_str() == "from"
                && matches!(&member.object, Expression::Identifier(id) if id.name == "Array")
            {
                return call
                    .arguments
                    .first()
                    .and_then(Argument::as_expression)
                    .and_then(source_array);
            }
            matches!(
                member.property.name.as_str(),
                "map"
                    | "filter"
                    | "flatMap"
                    | "flat"
                    | "slice"
                    | "sort"
                    | "toSorted"
                    | "reverse"
                    | "splice"
            )
            .then(|| source_array(&member.object))
            .flatten()
        }
        _ => None,
    }
}

include!("array_uses.rs");

#[cfg(all(test, feature = "test-instrumentation"))]
pub(super) fn callback_summaries() -> usize {
    callback_aliases::last_summaries()
}
