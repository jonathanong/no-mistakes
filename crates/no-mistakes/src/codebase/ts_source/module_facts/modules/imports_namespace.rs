use super::*;
use oxc_syntax::symbol::SymbolId;

pub(super) fn runtime_members(semantic: &Semantic<'_>, symbol: SymbolId) -> Option<Vec<String>> {
    let nodes = semantic.nodes();
    let mut members = Vec::new();
    for reference in semantic.scoping().get_resolved_references(symbol) {
        if reference.is_write() || reference.is_type() {
            return None;
        }
        let node = reference.node_id();
        let span = nodes.kind(node).span();
        let member = nodes.parent_id(node);
        let name = match nodes.kind(member) {
            AstKind::StaticMemberExpression(value) if value.object.span() == span => {
                value.property.name.to_string()
            }
            AstKind::ComputedMemberExpression(value) if value.object.span() == span => {
                let Expression::StringLiteral(key) = &value.expression else {
                    return None;
                };
                crate::ast::js_literal(key).into_owned()
            }
            _ => return None,
        };
        let writes_member = match nodes.parent_kind(member) {
            AstKind::AssignmentExpression(value) => value.left.span() == nodes.kind(member).span(),
            AstKind::ForInStatement(value) => value
                .left
                .span()
                .contains_inclusive(nodes.kind(member).span()),
            AstKind::ForOfStatement(value) => value
                .left
                .span()
                .contains_inclusive(nodes.kind(member).span()),
            AstKind::AssignmentTargetPropertyProperty(value) => value
                .binding
                .span()
                .contains_inclusive(nodes.kind(member).span()),
            AstKind::AssignmentTargetWithDefault(value) => value
                .binding
                .span()
                .contains_inclusive(nodes.kind(member).span()),
            AstKind::ArrayAssignmentTarget(_)
            | AstKind::AssignmentTargetRest(_)
            | AstKind::UpdateExpression(_) => true,
            AstKind::UnaryExpression(value) => {
                value.operator == oxc_syntax::operator::UnaryOperator::Delete
            }
            _ => false,
        };
        if writes_member {
            return None;
        }
        members.push(name);
    }
    members.sort_unstable();
    members.dedup();
    (!members.is_empty()).then_some(members)
}
