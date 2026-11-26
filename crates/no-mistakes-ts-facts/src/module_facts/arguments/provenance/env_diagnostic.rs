use super::super::Projector;
use oxc_ast::{ast::Expression, AstKind};
use oxc_syntax::node::NodeId;

pub(super) fn cleared(projector: &Projector<'_, '_>, node: NodeId) -> bool {
    let expression = match projector.semantic.nodes().kind(node) {
        AstKind::UnaryExpression(_) => return true,
        AstKind::CallExpression(call) => call
            .arguments
            .get(1)
            .and_then(|value| value.as_expression()),
        _ => None,
    };
    expression.is_some_and(|expression| {
        let Expression::Identifier(id) = expression.get_inner_expression() else {
            return false;
        };
        id.name == "undefined"
            && id.reference_id.get().is_some_and(|reference| {
                projector
                    .semantic
                    .scoping()
                    .get_reference(reference)
                    .symbol_id()
                    .is_none()
            })
    })
}
