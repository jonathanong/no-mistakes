#[path = "pass_through_effects.rs"]
mod effects;
#[path = "pass_through_imports.rs"]
mod imports;
use super::provenance::pass_through::{assignment_parent, forwards_mock_arguments};
use crate::fx::{FxHashMap, FxHashSet};
use oxc_ast::{ast::*, AstKind};
use oxc_semantic::Semantic;
use oxc_syntax::symbol::SymbolId;

pub(crate) fn transparent_receiver_writes(semantic: &Semantic<'_>) -> FxHashSet<u32> {
    if !semantic.source_text().contains("...") || !semantic.source_text().contains('=') {
        return Default::default();
    }
    let unique = imports::unique_receivers(semantic);
    let calls = effects::calls_per_function(semantic);
    let mut wrappers: FxHashMap<(SymbolId, String), FxHashSet<u32>> = FxHashMap::default();
    for node in semantic.nodes().iter() {
        let AstKind::CallExpression(call) = node.kind() else {
            continue;
        };
        let Some(name) = forwards_mock_arguments(semantic, node.id(), call) else {
            continue;
        };
        let Some(function) = semantic.nodes().ancestors(node.id()).find(|node| {
            matches!(
                node.kind(),
                AstKind::Function(_) | AstKind::ArrowFunctionExpression(_)
            )
        }) else {
            continue;
        };
        if calls.get(&function.id()) != Some(&1) {
            continue;
        }
        let Some(parent) = assignment_parent(semantic, function.id()) else {
            continue;
        };
        let AstKind::AssignmentExpression(assignment) = semantic.nodes().kind(parent) else {
            continue;
        };
        let AssignmentTarget::StaticMemberExpression(member) = &assignment.left else {
            continue;
        };
        let Some(symbol) = object_symbol(semantic, &member.object) else {
            continue;
        };
        if !unique.contains(&symbol) {
            continue;
        }
        wrappers
            .entry((symbol, name))
            .or_default()
            .insert(assignment.span.start);
    }
    let mut result = FxHashSet::default();
    for ((symbol, property), offsets) in wrappers {
        if let Some(writes) = member_writes(semantic, symbol, &property, &offsets) {
            result.extend(writes);
        }
    }
    result
}

fn object_symbol(semantic: &Semantic<'_>, object: &Expression<'_>) -> Option<SymbolId> {
    let Expression::Identifier(id) = object.get_inner_expression() else {
        return None;
    };
    semantic
        .scoping()
        .get_reference(id.reference_id.get()?)
        .symbol_id()
}

fn member_writes(
    semantic: &Semantic<'_>,
    symbol: SymbolId,
    property: &str,
    wrappers: &FxHashSet<u32>,
) -> Option<FxHashSet<u32>> {
    let mut writes = wrappers.clone();
    for (index, reference) in semantic
        .scoping()
        .get_resolved_references(symbol)
        .enumerate()
    {
        if !reference.flags().is_value() || reference.flags().is_value_as_type() {
            continue;
        }
        if index >= 256 || reference.flags().is_write() {
            return None;
        }
        let member = semantic.nodes().parent_node(reference.node_id());
        let AstKind::StaticMemberExpression(access) = member.kind() else {
            return None;
        };
        if access.property.name != property {
            continue;
        }
        match semantic.nodes().parent_kind(member.id()) {
            AstKind::AssignmentExpression(assignment) if matches!(&assignment.left, AssignmentTarget::StaticMemberExpression(left) if left.span == access.span) =>
            {
                if !wrappers.contains(&assignment.span.start)
                    && !restores_capture(semantic, assignment, symbol, property)
                {
                    return None;
                }
                writes.insert(assignment.span.start);
            }
            AstKind::CallExpression(call) if matches!(call.callee.get_inner_expression(), Expression::StaticMemberExpression(callee) if callee.span == access.span) =>
                {}
            AstKind::VariableDeclarator(capture) if matches!(&capture.id, BindingPattern::BindingIdentifier(id) if id.symbol_id.get().is_some_and(|symbol| semantic.scoping().symbol_flags(symbol).is_const_variable())) =>
                {}
            _ => return None,
        }
    }
    Some(writes)
}

fn restores_capture(
    semantic: &Semantic<'_>,
    assignment: &AssignmentExpression<'_>,
    object: SymbolId,
    property: &str,
) -> bool {
    if assignment.operator != AssignmentOperator::Assign {
        return false;
    }
    let Expression::Identifier(id) = assignment.right.get_inner_expression() else {
        return false;
    };
    let Some(symbol) = object_symbol(semantic, &assignment.right) else {
        return false;
    };
    if !semantic.scoping().symbol_flags(symbol).is_const_variable()
        || semantic
            .scoping()
            .get_resolved_references(symbol)
            .any(|r| r.flags().is_write())
    {
        return false;
    }
    let AstKind::VariableDeclarator(capture) = semantic
        .nodes()
        .kind(semantic.scoping().symbol_declaration(symbol))
    else {
        return false;
    };
    if !matches!(&capture.id, BindingPattern::BindingIdentifier(id) if id.symbol_id.get() == Some(symbol))
        || capture.span.end >= id.span.start
    {
        return false;
    }
    matches!(capture.init.as_ref().map(Expression::get_inner_expression), Some(Expression::StaticMemberExpression(member)) if member.property.name == property && object_symbol(semantic, &member.object) == Some(object))
}

pub(crate) fn transparent_receiver_writes_from_program(
    program: &Program<'_>,
) -> std::sync::Arc<FxHashSet<u32>> {
    if !program.source_text.contains("...")
        || !program.source_text.contains('=')
        || !program.body.iter().any(|statement| match statement {
            Statement::ImportDeclaration(import) => {
                let source = crate::ast::js_literal(&import.source);
                matches!(
                    source.strip_prefix("node:").unwrap_or(&source),
                    "fs" | "fs/promises" | "fs-extra"
                )
            }
            _ => false,
        })
    {
        return Default::default();
    }
    let built = oxc_semantic::SemanticBuilder::new()
        .with_build_nodes(true)
        .build(program);
    std::sync::Arc::new(transparent_receiver_writes(&built.semantic))
}
