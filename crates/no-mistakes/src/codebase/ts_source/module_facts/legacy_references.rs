//! Preserve 0.153 reference metadata without changing the semantic evaluator.
use super::BindingReference;
use crate::fx::FxHashMap;
use oxc_ast::AstKind;
use oxc_semantic::{ReferenceFlags, ReferenceId, Semantic};
use oxc_span::{GetSpan, Span};
use oxc_syntax::{scope::ScopeId, symbol::SymbolId};

pub(super) fn collect(
    semantic: &Semantic<'_>,
) -> FxHashMap<SymbolId, Vec<(ReferenceId, BindingReference)>> {
    let mut projected = FxHashMap::default();
    let scoping = semantic.scoping();
    let name = oxc_str::Ident::from("arguments");
    let Some(unresolved) = scoping.root_unresolved_references().get(&name) else {
        return projected;
    };
    for &id in unresolved {
        let reference = scoping.get_reference(id);
        for scope in scoping.scope_ancestors(reference.scope_id()) {
            let Some(symbol) = scoping.get_binding(scope, name) else {
                continue;
            };
            if body_binding_hidden(semantic, scope, symbol, reference.node_id()) {
                continue;
            }
            let flags = scoping.symbol_flags(symbol);
            let mut usage = reference.flags();
            if (usage.is_namespace()
                && !usage.is_value_as_type()
                && !flags.can_be_referenced_as_namespace())
                || !((usage.is_value() && flags.can_be_referenced_by_value())
                    || (usage.is_type() && flags.can_be_referenced_by_type())
                    || (usage.is_value_as_type() && flags.can_be_referenced_by_value_as_type()))
            {
                continue;
            }
            // This is OXC 0.153's post-resolution flag projection. Keep the
            // current semantic references untouched for actual arguments lookup.
            if (flags.is_value() && usage.is_value()) || (usage.is_namespace() && usage.is_read()) {
                usage -= ReferenceFlags::Type;
            } else {
                usage = ReferenceFlags::Type;
            }
            projected.entry(symbol).or_insert_with(Vec::new).push((
                id,
                BindingReference {
                    span: semantic.nodes().kind(reference.node_id()).span().into(),
                    runtime: usage.is_value() && !usage.is_value_as_type(),
                    type_only: usage.is_type() || usage.is_value_as_type(),
                },
            ));
            break;
        }
    }
    for references in projected.values_mut() {
        references.sort_unstable_by_key(|(id, _)| *id);
    }
    projected
}

fn body_binding_hidden(
    semantic: &Semantic<'_>,
    scope: ScopeId,
    symbol: SymbolId,
    reference: oxc_syntax::node::NodeId,
) -> bool {
    let scoping = semantic.scoping();
    let body = match semantic.nodes().kind(scoping.get_node_id(scope)) {
        AstKind::Function(function) => function.body.as_ref().map(|body| body.span),
        AstKind::ArrowFunctionExpression(arrow) => Some(arrow.body.span()),
        _ => None,
    };
    let reference = semantic.nodes().kind(reference).span();
    // Parameter initializers resolve before body declarations exist. A named
    // function expression or parameter already exists and remains visible.
    body.is_some_and(|body| {
        !contains(body, reference) && contains(body, scoping.symbol_span(symbol))
    })
}

fn contains(outer: Span, inner: Span) -> bool {
    outer.start <= inner.start && inner.end <= outer.end
}

pub(super) fn merge(
    semantic: &Semantic<'_>,
    symbol: SymbolId,
    references: &mut Vec<BindingReference>,
    extra: Vec<(ReferenceId, BindingReference)>,
) {
    let original = semantic.scoping().get_resolved_reference_ids(symbol);
    for (offset, (id, reference)) in extra.into_iter().enumerate() {
        let position = original.partition_point(|original| *original < id) + offset;
        references.insert(position, reference);
    }
}

#[cfg(test)]
mod tests;
