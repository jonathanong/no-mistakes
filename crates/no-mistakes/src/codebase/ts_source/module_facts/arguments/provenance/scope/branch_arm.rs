//! A named branch is one statement in an `if` or conditional arm.
use oxc_ast::ast::{ConditionalExpression, IfStatement};
use oxc_ast::AstKind;
use oxc_span::GetSpan;
use oxc_syntax::node::NodeId;

pub(super) fn in_named_branch(nodes: &oxc_semantic::AstNodes<'_>, node: NodeId) -> bool {
    let current = skip_wrappers(nodes, node);
    if direct_arm(nodes, current) {
        return true;
    }
    // The write itself must be the statement. An argument is not.
    let statement = nodes.parent_id(current);
    if !matches!(nodes.kind(statement), AstKind::ExpressionStatement(_)) {
        return false;
    }
    // A hook callback is a statement in the arm only after leaving the call.
    let mut current = statement;
    loop {
        let parent = nodes.parent_id(current);
        match nodes.kind(parent) {
            AstKind::BlockStatement(_)
            | AstKind::Function(_)
            | AstKind::ArrowFunctionExpression(_)
            | AstKind::CallExpression(_)
            | AstKind::ExpressionStatement(_)
            | AstKind::ParenthesizedExpression(_) => current = parent,
            AstKind::IfStatement(stmt) => return is_if_arm(nodes, stmt, current),
            AstKind::ConditionalExpression(expr) => {
                return is_conditional_arm(nodes, expr, current)
            }
            _ => return false,
        }
    }
}

fn skip_wrappers(nodes: &oxc_semantic::AstNodes<'_>, mut current: NodeId) -> NodeId {
    loop {
        let parent = nodes.parent_id(current);
        if !matches!(
            nodes.kind(parent),
            AstKind::ParenthesizedExpression(_)
                | AstKind::TSAsExpression(_)
                | AstKind::TSSatisfiesExpression(_)
                | AstKind::TSNonNullExpression(_)
                | AstKind::TSTypeAssertion(_)
                | AstKind::ChainExpression(_)
        ) {
            return current;
        }
        current = parent;
    }
}

fn direct_arm(nodes: &oxc_semantic::AstNodes<'_>, current: NodeId) -> bool {
    match nodes.kind(nodes.parent_id(current)) {
        AstKind::IfStatement(stmt) => is_if_arm(nodes, stmt, current),
        AstKind::ConditionalExpression(expr) => is_conditional_arm(nodes, expr, current),
        _ => false,
    }
}

fn is_if_arm(nodes: &oxc_semantic::AstNodes<'_>, stmt: &IfStatement<'_>, current: NodeId) -> bool {
    let span = nodes.kind(current).span();
    span == stmt.consequent.span()
        || stmt
            .alternate
            .as_ref()
            .is_some_and(|alt| alt.span() == span)
}

fn is_conditional_arm(
    nodes: &oxc_semantic::AstNodes<'_>,
    expr: &ConditionalExpression<'_>,
    current: NodeId,
) -> bool {
    let span = nodes.kind(current).span();
    expr.consequent.span() == span || expr.alternate.span() == span
}
