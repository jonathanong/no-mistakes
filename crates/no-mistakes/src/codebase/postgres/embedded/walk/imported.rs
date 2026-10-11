use super::ScopeVisitor;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::Expression;

/// Keep only a stable, zero-argument helper reference. Module identity and
/// exports are checked against the prepared import facts after resolution.
pub(super) fn unresolved_helper_name(
    expression: &Expression<'_>,
    visitor: &ScopeVisitor<'_>,
) -> Option<String> {
    match unwrap_ts_wrappers(expression) {
        Expression::Identifier(name) => visitor.lookup(name.name.as_str())?.pending_imported_helper,
        Expression::CallExpression(call) if !call.optional && call.arguments.is_empty() => {
            let Expression::Identifier(name) = unwrap_ts_wrappers(&call.callee) else {
                return None;
            };
            let name = name.name.as_str();
            (!visitor.shadowed_locally(name) && !visitor.functions.has_static_helper(name))
                .then(|| name.to_string())
        }
        _ => None,
    }
}
