use crate::codebase::ts_source::module_facts::RequireBinding;
use crate::fx::{FxHashMap, FxHashSet};
use oxc_ast::{ast::*, AstKind};
use oxc_semantic::{Semantic, SymbolId};
use oxc_span::GetSpan;

pub(super) enum Selection {
    Named(bool),
    Module(bool),
    Unknown,
}

pub(super) struct DirectUses {
    pub(super) sync_unsafe: bool,
    pub(super) promises_unsafe: bool,
}

pub(super) fn selection(
    semantic: &Semantic<'_>,
    symbol: SymbolId,
    binding: &RequireBinding,
    load_end: u32,
    specifier: &str,
) -> Selection {
    let AstKind::VariableDeclarator(declaration) = semantic
        .nodes()
        .kind(semantic.scoping().symbol_declaration(symbol))
    else {
        return Selection::Unknown;
    };
    let Some(mut expression) = declaration
        .init
        .as_ref()
        .map(Expression::get_inner_expression)
    else {
        return Selection::Unknown;
    };
    let mut members = Vec::new();
    while let Expression::StaticMemberExpression(member) = expression {
        if members.len() == 3 {
            return Selection::Unknown;
        }
        members.push(member.property.name.as_str());
        expression = member.object.get_inner_expression();
    }
    let Expression::CallExpression(call) = expression else {
        return Selection::Unknown;
    };
    if call.span.start != binding.load_start || call.span.end != load_end {
        return Selection::Unknown;
    }
    members.reverse();
    if let Some(imported) = binding.imported.as_deref() {
        members.push(imported);
    }
    let sync = matches!(specifier, "fs" | "node:fs");
    match members.as_slice() {
        [] => Selection::Module(sync),
        ["promises"] if sync => Selection::Module(false),
        ["mkdtemp"] => Selection::Named(sync),
        ["mkdtempSync"] if sync => Selection::Named(true),
        ["promises", "mkdtemp"] if sync => Selection::Named(false),
        _ => Selection::Unknown,
    }
}

pub(super) fn direct_writes(
    semantic: &Semantic<'_>,
    loads: &FxHashMap<u32, (u32, &str)>,
    tracked: &FxHashSet<u32>,
) -> DirectUses {
    let nodes = semantic.nodes();
    let called_members = called_member_loads(semantic, loads);
    let mut sync_unsafe = false;
    let mut promises_unsafe = false;
    for node in nodes.iter() {
        let AstKind::CallExpression(call) = node.kind() else {
            continue;
        };
        let Some(&(load_end, specifier)) = loads.get(&call.span.start) else {
            continue;
        };
        if call.span.end != load_end {
            continue;
        }
        let sync = matches!(specifier, "fs" | "node:fs");
        let member = nodes.parent_id(node.id());
        let Some(name) = member_name(nodes, member, call.span) else {
            if !called_members.contains(&(call.span.start, call.span.end))
                && !tracked_initializer(nodes, node.id(), tracked.contains(&call.span.start))
                && !super::safety::safe_nonfactory_spy(semantic, node.id())
            {
                if sync {
                    sync_unsafe = true;
                } else {
                    promises_unsafe = true;
                }
            }
            continue;
        };
        if sync && name == "promises" {
            if !called_members.contains(&(call.span.start, call.span.end))
                && !super::safety::promises_member_safe(nodes, member)
                && !tracked_initializer(nodes, member, tracked.contains(&call.span.start))
                && !super::safety::safe_nonfactory_spy(semantic, member)
            {
                promises_unsafe = true;
            }
        } else if super::super::super::super::modules::imports_namespace::writes_member(
            nodes, member,
        ) && matches!(name.as_str(), "mkdtemp" | "mkdtempSync")
        {
            if sync {
                sync_unsafe = true;
            } else {
                promises_unsafe = true;
            }
        }
    }
    DirectUses {
        sync_unsafe,
        promises_unsafe,
    }
}

fn called_member_loads(
    semantic: &Semantic<'_>,
    loads: &FxHashMap<u32, (u32, &str)>,
) -> FxHashSet<(u32, u32)> {
    semantic
        .nodes()
        .iter()
        .filter_map(|node| {
            let AstKind::CallExpression(call) = node.kind() else {
                return None;
            };
            let mut expression = call.callee.get_inner_expression();
            let mut member = false;
            loop {
                expression = match expression {
                    Expression::StaticMemberExpression(value) => &value.object,
                    Expression::ComputedMemberExpression(value) => &value.object,
                    Expression::CallExpression(load) if member => {
                        return loads
                            .get(&load.span.start)
                            .is_some_and(|(end, _)| *end == load.span.end)
                            .then_some((load.span.start, load.span.end));
                    }
                    _ => return None,
                }
                .get_inner_expression();
                member = true;
            }
        })
        .collect()
}

fn tracked_initializer(
    nodes: &oxc_semantic::AstNodes<'_>,
    node: oxc_semantic::NodeId,
    tracked: bool,
) -> bool {
    tracked
        && matches!(nodes.parent_kind(node), AstKind::VariableDeclarator(declaration)
            if declaration.init.as_ref().is_some_and(|init| init.span() == nodes.kind(node).span()))
}

fn member_name(
    nodes: &oxc_semantic::AstNodes<'_>,
    member: oxc_semantic::NodeId,
    object: oxc_span::Span,
) -> Option<String> {
    match nodes.kind(member) {
        AstKind::StaticMemberExpression(value) if value.object.span() == object => {
            Some(value.property.name.to_string())
        }
        AstKind::ComputedMemberExpression(value) if value.object.span() == object => {
            let Expression::StringLiteral(key) = &value.expression else {
                return None;
            };
            Some(crate::ast::js_literal(key).into_owned())
        }
        _ => None,
    }
}
