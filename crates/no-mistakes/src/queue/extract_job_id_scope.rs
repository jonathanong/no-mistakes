use super::QueueVisitor;
use crate::queue::extract_helpers::static_string;
use oxc_ast::ast::{BindingPattern, Expression, IdentifierReference, VariableDeclarator};
use oxc_syntax::symbol::SymbolId;

pub(in crate::queue) fn note_const(visitor: &mut QueueVisitor<'_>, decl: &VariableDeclarator<'_>) {
    let (Some(scoping), BindingPattern::BindingIdentifier(id), Some(init)) =
        (visitor.scoping, &decl.id, &decl.init)
    else {
        return;
    };
    if let Some(symbol) = id.symbol_id.get() {
        if scoping.symbol_flags(symbol).is_const_variable() {
            if let Some(value) = static_string(init.get_inner_expression()) {
                visitor.job_id_consts.insert(symbol, value);
            }
        }
    }
}

pub(in crate::queue) fn note_flow_binding(
    visitor: &mut QueueVisitor<'_>,
    pattern: &BindingPattern<'_>,
) {
    if let Some(symbol) = stable_binding(visitor, pattern) {
        visitor.job_id_flows.insert(symbol);
    }
}

pub(in crate::queue) fn note_queue_binding(
    visitor: &mut QueueVisitor<'_>,
    pattern: &BindingPattern<'_>,
) {
    if let Some(symbol) = stable_binding(visitor, pattern) {
        visitor.job_id_queues.insert(symbol);
    }
}

fn stable_binding(visitor: &QueueVisitor<'_>, pattern: &BindingPattern<'_>) -> Option<SymbolId> {
    let BindingPattern::BindingIdentifier(id) = pattern else {
        return None;
    };
    let symbol = id.symbol_id.get()?;
    (!visitor.scoping?.symbol_is_mutated(symbol)).then_some(symbol)
}

pub(in crate::queue) fn referenced_symbol(
    visitor: &QueueVisitor<'_>,
    id: &IdentifierReference<'_>,
) -> Option<SymbolId> {
    visitor
        .scoping?
        .get_reference(id.reference_id.get()?)
        .symbol_id()
}

pub(super) fn imported(visitor: &QueueVisitor<'_>, symbol: SymbolId) -> bool {
    visitor.scoping.is_some_and(|scoping| {
        let flags = scoping.symbol_flags(symbol);
        flags.is_import() && !flags.is_type_import()
    })
}

pub(in crate::queue) fn queue_identifier(
    visitor: &QueueVisitor<'_>,
    id: &IdentifierReference<'_>,
) -> bool {
    referenced_symbol(visitor, id)
        .is_some_and(|symbol| visitor.job_id_queues.contains(&symbol) || imported(visitor, symbol))
}

pub(in crate::queue) fn is_imported_callee(
    visitor: &QueueVisitor<'_>,
    callee: &Expression<'_>,
) -> bool {
    matches!(callee.get_inner_expression(), Expression::Identifier(id)
        if referenced_symbol(visitor, id).is_some_and(|symbol| imported(visitor, symbol)))
}
