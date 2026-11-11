use super::*;
use oxc_syntax::symbol::SymbolId;

pub(super) fn runtime_members(
    semantic: &Semantic<'_>,
    symbol: SymbolId,
    calls: &mut Vec<String>,
) -> Option<Vec<String>> {
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
        if writes_member(nodes, member) {
            return None;
        }
        if receiver_use(nodes, member) {
            calls.push(name.clone());
        }
        members.push(name);
    }
    members.sort_unstable();
    members.dedup();
    (!members.is_empty()).then_some(members)
}

pub(super) fn receiver_use(
    nodes: &oxc_semantic::AstNodes<'_>,
    member: oxc_semantic::NodeId,
) -> bool {
    let span = nodes.kind(member).span();
    nodes
        .ancestors(member)
        .find_map(|node| match node.kind() {
            AstKind::ParenthesizedExpression(_)
            | AstKind::TSAsExpression(_)
            | AstKind::TSSatisfiesExpression(_)
            | AstKind::TSNonNullExpression(_)
            | AstKind::TSInstantiationExpression(_)
            | AstKind::TSTypeAssertion(_)
            | AstKind::ChainExpression(_) => None,
            AstKind::CallExpression(value) => Some(value.callee.span().contains_inclusive(span)),
            AstKind::TaggedTemplateExpression(value) => {
                Some(value.tag.span().contains_inclusive(span))
            }
            _ => Some(false),
        })
        .unwrap_or(false)
}

pub(in crate::codebase::ts_source::module_facts) fn writes_member(
    nodes: &oxc_semantic::AstNodes<'_>,
    member: oxc_semantic::NodeId,
) -> bool {
    match nodes.parent_kind(member) {
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
    }
}
