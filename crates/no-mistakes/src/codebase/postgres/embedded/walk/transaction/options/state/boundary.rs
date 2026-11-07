use super::{unwrap_ts_wrappers, Expression, ScopeVisitor};
use oxc_ast::ast::{CallExpression, IdentifierReference};

pub(in crate::codebase::postgres::embedded::walk::transaction) fn end_alias(
    visitor: &mut ScopeVisitor<'_>,
    id: &IdentifierReference<'_>,
    call: &CallExpression<'_>,
) -> bool {
    let Some(identity) = visitor
        .lookup(id.name.as_str())
        .and_then(|binding| binding.value_identity)
    else {
        return false;
    };
    let Some(captured) = visitor.transaction_options.aliases.get(&identity) else {
        return false;
    };
    if captured.callable && !matches!(unwrap_ts_wrappers(&call.callee), Expression::Identifier(_)) {
        return false;
    }
    let identity = visitor.scopes.iter_mut().rev().find_map(|scope| {
        let binding = scope.get_mut(&captured.name)?;
        (binding.value_identity == Some(captured.identity))
            .then(|| binding.value_identity.take())
            .flatten()
    });
    super::super::super::super::transaction_flow::complete_boundary(visitor, call, identity);
    true
}

pub(super) fn mutate(visitor: &mut ScopeVisitor<'_>, expression: &Expression<'_>) {
    let Expression::Identifier(id) = unwrap_ts_wrappers(expression) else {
        return;
    };
    let Some(identity) = visitor
        .lookup(id.name.as_str())
        .and_then(|binding| binding.value_identity)
    else {
        return;
    };
    let origin = visitor
        .transaction_options
        .aliases
        .get(&identity)
        .map(|captured| captured.identity)
        .unwrap_or(identity);
    visitor.transaction_options.mutated.insert(origin);
}

pub(in crate::codebase::postgres::embedded::walk::transaction) fn callable(
    visitor: &ScopeVisitor<'_>,
    expression: &Expression<'_>,
) -> bool {
    if visitor.transaction_options.aliases.is_empty() {
        return false;
    }
    let Expression::Identifier(id) = unwrap_ts_wrappers(expression) else {
        return false;
    };
    visitor
        .lookup(id.name.as_str())
        .and_then(|binding| binding.value_identity)
        .and_then(|identity| visitor.transaction_options.aliases.get(&identity))
        .is_some_and(|captured| captured.callable)
}
