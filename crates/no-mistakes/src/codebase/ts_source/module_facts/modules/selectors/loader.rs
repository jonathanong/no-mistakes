use super::*;

/// `dynamic(() => import("x"))`: only an arrow passed directly as a call
/// argument is a loader whose result is the default export. A stored loader
/// (`const load = () => import("x")`) can be read in any way, so it is unproven.
pub(super) fn default(
    arrow: NodeId,
    function: &ArrowFunctionExpression<'_>,
    semantic: &Semantic<'_>,
) -> Option<Vec<String>> {
    let (call_id, AstKind::CallExpression(call)) = parent(arrow, semantic)? else {
        return None;
    };
    if !function.is_expression() || !call.arguments.iter().any(|a| a.span() == function.span) {
        return None;
    }
    // The wrapper's own result may be unwrapped, chained or handed to an unknown
    // consumer, so the module is not provably its default.
    let kind = parent(call_id, semantic)?.1;
    if consumed(kind) {
        return None;
    }
    match kind {
        // Returned (`return wrap(...)`, `() => wrap(...)`) to an unknown consumer.
        AstKind::ReturnStatement(_) | AstKind::ArrowFunctionExpression(_) => None,
        AstKind::VariableDeclarator(declarator) => {
            let BindingPattern::BindingIdentifier(binding) = &declarator.id else {
                return None;
            };
            let unconsumed = semantic
                .scoping()
                .get_resolved_references(binding.symbol_id.get()?)
                .all(|reference| {
                    parent(reference.node_id(), semantic).is_some_and(|p| !consumed(p.1))
                });
            unconsumed.then(|| vec!["default".to_string()])
        }
        _ => Some(vec!["default".to_string()]),
    }
}

/// A parent that reads, chains or forwards the wrapper's result.
fn consumed(kind: AstKind<'_>) -> bool {
    matches!(
        kind,
        AstKind::StaticMemberExpression(_)
            | AstKind::ComputedMemberExpression(_)
            | AstKind::PrivateFieldExpression(_)
            | AstKind::ChainExpression(_)
            | AstKind::AwaitExpression(_)
            | AstKind::CallExpression(_)
            | AstKind::NewExpression(_)
    )
}
