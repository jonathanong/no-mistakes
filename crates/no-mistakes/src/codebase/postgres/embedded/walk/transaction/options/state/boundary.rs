use super::{unwrap_ts_wrappers, Expression, ScopeVisitor};

pub(in crate::codebase::postgres::embedded::walk::transaction) fn end_alias(
    visitor: &mut ScopeVisitor<'_>,
    expression: &Expression<'_>,
) -> bool {
    let Expression::Identifier(id) = unwrap_ts_wrappers(expression) else {
        return false;
    };
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
