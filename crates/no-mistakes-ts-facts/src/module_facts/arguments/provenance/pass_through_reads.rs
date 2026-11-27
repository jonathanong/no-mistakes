use oxc_ast::{ast::*, AstKind};
use oxc_semantic::{NodeId, Semantic};
use oxc_syntax::{operator::BinaryOperator, symbol::SymbolId};

pub(super) fn unchanged_arguments(
    semantic: &Semantic<'_>,
    function: NodeId,
    symbol: SymbolId,
    forwarded: &IdentifierReference<'_>,
) -> bool {
    semantic.scoping().get_resolved_references(symbol).enumerate().all(|(index, reference)| {
        if index >= 256 || reference.flags().is_write() {
            return false;
        }
        let node = reference.node_id();
        if matches!(semantic.nodes().kind(node), AstKind::IdentifierReference(id) if id.span == forwarded.span) {
            return true;
        }
        let owner = semantic.nodes().ancestors(node).find(|ancestor| matches!(ancestor.kind(), AstKind::Function(_) | AstKind::ArrowFunctionExpression(_)));
        owner.is_some_and(|owner| owner.id() == function) && readonly_member(semantic, node)
    })
}

fn readonly_member(semantic: &Semantic<'_>, reference: NodeId) -> bool {
    let member = semantic.nodes().parent_node(reference);
    let inspected = match member.kind() {
        AstKind::ComputedMemberExpression(member) => matches!(
            member.expression.get_inner_expression(),
            Expression::NumericLiteral(_)
        ),
        AstKind::StaticMemberExpression(member) => member.property.name == "length",
        _ => false,
    };
    inspected
        && match semantic.nodes().parent_kind(member.id()) {
            AstKind::BinaryExpression(binary) => matches!(
                binary.operator,
                BinaryOperator::StrictEquality | BinaryOperator::StrictInequality
            ),
            AstKind::IfStatement(statement) => matches!(
                statement.test.get_inner_expression(),
                Expression::ComputedMemberExpression(_) | Expression::StaticMemberExpression(_)
            ),
            _ => false,
        }
}
