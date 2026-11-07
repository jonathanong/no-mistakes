use super::{unwrap_ts_wrappers, Expression, ScopeVisitor};
use oxc_ast::ast::IdentifierReference;

pub(in crate::codebase::postgres::embedded::walk::transaction) fn end_alias(
    visitor: &mut ScopeVisitor<'_>,
    id: &IdentifierReference<'_>,
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
    for scope in visitor.scopes.iter_mut().rev() {
        if let Some(binding) = scope.get_mut(&captured.name) {
            if binding.value_identity == Some(captured.identity) {
                binding.value_identity = None;
                return true;
            }
        }
    }
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
