use super::TempFactoryBinding;
use crate::fx::FxHashMap;
use oxc_ast::{ast::*, AstKind};
use oxc_semantic::{Semantic, SymbolId};
use oxc_span::GetSpan;

pub(super) fn record_module(
    semantic: &Semantic<'_>,
    symbol: SymbolId,
    id: usize,
    sync: bool,
    bindings: &mut FxHashMap<usize, TempFactoryBinding>,
    unsafe_family: &mut bool,
) {
    if factory_view_safe(semantic, symbol, sync) {
        bindings.insert(
            id,
            TempFactoryBinding::Module {
                sync,
                promises_safe: true,
            },
        );
    } else if semantic
        .scoping()
        .get_resolved_references(symbol)
        .next()
        .is_some()
    {
        *unsafe_family = true;
    }
}

fn factory(name: &str) -> bool {
    matches!(name, "mkdtemp" | "mkdtempSync")
}

pub(super) fn factory_view_safe(semantic: &Semantic<'_>, symbol: SymbolId, sync: bool) -> bool {
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
            if sync && name == "promises" && !promises_member_safe(nodes, parent) {
                return false;
            }
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

pub(super) fn promises_member_safe(
    nodes: &oxc_semantic::AstNodes<'_>,
    member: oxc_semantic::NodeId,
) -> bool {
    if super::super::super::super::modules::imports_namespace::writes_member(nodes, member) {
        return false;
    }
    let span = nodes.kind(member).span();
    let next = nodes.parent_id(member);
    let name = match nodes.kind(next) {
        AstKind::StaticMemberExpression(nested) if nested.object.span() == span => {
            nested.property.name.as_str()
        }
        AstKind::ComputedMemberExpression(nested) if nested.object.span() == span => {
            let Expression::StringLiteral(key) = &nested.expression else {
                return false;
            };
            key.value.as_str().unwrap_or_default()
        }
        _ => return false,
    };
    !factory(name)
        || !super::super::super::super::modules::imports_namespace::writes_member(nodes, next)
}

pub(super) fn safe_nonfactory_spy(semantic: &Semantic<'_>, node: oxc_semantic::NodeId) -> bool {
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
    if factory(&name) || name == "promises" {
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
