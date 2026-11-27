use oxc_ast::{ast::UnaryOperator, AstKind};
use oxc_span::GetSpan;

pub(super) fn member_assignment_target(
    nodes: &oxc_semantic::AstNodes<'_>,
    member: oxc_semantic::NodeId,
) -> bool {
    let span = nodes.kind(member).span();
    nodes
        .ancestors(member)
        .find_map(|ancestor| match ancestor.kind() {
            AstKind::StaticMemberExpression(member)
                if member.object.span().contains_inclusive(span) =>
            {
                None
            }
            AstKind::ComputedMemberExpression(member)
                if member.object.span().contains_inclusive(span) =>
            {
                None
            }
            AstKind::UpdateExpression(update) => {
                Some(update.argument.span().contains_inclusive(span))
            }
            AstKind::UnaryExpression(unary) => Some(
                unary.operator == UnaryOperator::Delete
                    && unary.argument.span().contains_inclusive(span),
            ),
            AstKind::CallExpression(call) => Some(call.callee.span().contains_inclusive(span)),
            AstKind::NewExpression(call) => Some(call.callee.span().contains_inclusive(span)),
            AstKind::AssignmentExpression(assignment) => {
                Some(assignment.left.span().contains_inclusive(span))
            }
            AstKind::ForInStatement(statement) => {
                Some(statement.left.span().contains_inclusive(span))
            }
            AstKind::ForOfStatement(statement) => {
                Some(statement.left.span().contains_inclusive(span))
            }
            AstKind::AssignmentTargetPropertyProperty(property)
                if property.binding.span().contains_inclusive(span) =>
            {
                None
            }
            AstKind::AssignmentTargetWithDefault(default)
                if default.binding.span().contains_inclusive(span) =>
            {
                None
            }
            AstKind::ArrayAssignmentTarget(_)
            | AstKind::ObjectAssignmentTarget(_)
            | AstKind::AssignmentTargetRest(_)
            | AstKind::ParenthesizedExpression(_)
            | AstKind::TSAsExpression(_)
            | AstKind::TSTypeAssertion(_)
            | AstKind::TSNonNullExpression(_)
            | AstKind::TSSatisfiesExpression(_) => None,
            _ => Some(false),
        })
        .unwrap_or(false)
}
