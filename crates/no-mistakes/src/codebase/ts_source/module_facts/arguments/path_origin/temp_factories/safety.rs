use oxc_ast::{ast::*, AstKind};
use oxc_semantic::{Semantic, SymbolId};
use oxc_span::GetSpan;

fn factory(name: &str) -> bool {
    matches!(name, "mkdtemp" | "mkdtempSync")
}

pub(super) fn factory_view_safe(semantic: &Semantic<'_>, symbol: SymbolId) -> bool {
    let nodes = semantic.nodes();
    for reference in semantic.scoping().get_resolved_references(symbol) {
        if reference.is_write() {
            return false;
        }
        if reference.is_type() {
            continue;
        }
        let node = reference.node_id();
        let span = nodes.kind(node).span();
        let parent = nodes.parent_id(node);
        let member = match nodes.kind(parent) {
            AstKind::StaticMemberExpression(member) if member.object.span() == span => {
                Some(member.property.name.as_str())
            }
            AstKind::ComputedMemberExpression(member) if member.object.span() == span => {
                let Expression::StringLiteral(key) = &member.expression else {
                    return false;
                };
                Some(key.value.as_str().unwrap_or_default())
            }
            _ => None,
        };
        if let Some(name) = member {
            if factory(name)
                && super::super::super::super::modules::imports_namespace::writes_member(
                    nodes, parent,
                )
            {
                return false;
            }
            continue;
        }
        if !safe_nonfactory_spy(semantic, node) {
            return false;
        }
    }
    true
}

fn safe_nonfactory_spy(semantic: &Semantic<'_>, node: oxc_semantic::NodeId) -> bool {
    let nodes = semantic.nodes();
    let AstKind::CallExpression(call) = nodes.parent_kind(node) else {
        return false;
    };
    let Some(first) = call.arguments.first().and_then(Argument::as_expression) else {
        return false;
    };
    if first.span() != nodes.kind(node).span() {
        return false;
    }
    let Some(Expression::StringLiteral(key)) = call
        .arguments
        .get(1)
        .and_then(Argument::as_expression)
        .map(Expression::get_inner_expression)
    else {
        return false;
    };
    let name = crate::ast::js_literal(key);
    if factory(&name) {
        return false;
    }
    let Expression::StaticMemberExpression(member) = call.callee.get_inner_expression() else {
        return false;
    };
    if member.property.name != "spyOn" {
        return false;
    }
    let Expression::Identifier(object) = member.object.get_inner_expression() else {
        return false;
    };
    super::super::super::super::modules::mock_boundaries::mock_binding(object, semantic)
}
