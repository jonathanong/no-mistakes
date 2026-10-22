//! Bounded invocation-local literal facts; never evaluate source code.
use crate::fx::{FxHashMap, FxHashSet};
use oxc_ast::{ast::*, AstKind};
use oxc_semantic::Semantic;
use serde_json::Value;

mod aliases;
use aliases::AliasSafety;

#[cfg(test)]
#[macro_use]
pub(crate) mod probe;
#[cfg(not(test))]
#[macro_use]
mod probe_release;
#[cfg(test)]
pub(super) mod test_helpers;

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ArgumentValue {
    Scalar(Value),
    Array(Vec<Self>),
    Object(FxHashMap<String, Self>),
    Unknown,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ArgumentValues {
    pub(crate) values: Vec<ArgumentValue>,
    pub(crate) spread_from: Option<usize>,
}

pub(super) fn collect_selected(
    semantic: &Semantic<'_>,
    facts: &super::TypeScriptModuleFacts,
    offsets: Option<&std::collections::BTreeSet<u32>>,
) -> FxHashMap<u32, ArgumentValues> {
    let names: FxHashSet<_> = facts
        .exports
        .iter()
        .filter(|e| e.specifier.is_none() && !e.type_only)
        .map(|e| e.local.as_str())
        .collect();
    let roots: FxHashSet<_> = facts
        .scopes
        .iter()
        .filter(|s| s.parent_id.is_none())
        .map(|s| s.id)
        .collect();
    let exported: FxHashSet<_> = facts
        .bindings
        .iter()
        .filter(|b| roots.contains(&b.scope_id) && names.contains(b.name.as_str()))
        .map(|b| b.id)
        .collect();
    let mut immutable = AliasSafety::default();
    let mut result = FxHashMap::default();
    let mut dynamic_scope = false;
    for node in semantic.nodes().iter() {
        if matches!(node.kind(), AstKind::WithStatement(_)) {
            dynamic_scope = true;
        }
        if let AstKind::CallExpression(call) = node.kind() {
            if let Expression::Identifier(id) = call.callee.get_inner_expression() {
                if id.name == "eval"
                    && id
                        .reference_id
                        .get()
                        .is_some_and(|r| semantic.scoping().get_reference(r).symbol_id().is_none())
                {
                    dynamic_scope = true;
                }
            }
        }
        let (offset, arguments) = match node.kind() {
            AstKind::CallExpression(call) => (call.span.start, &call.arguments),
            AstKind::NewExpression(call) => (call.span.start, &call.arguments),
            _ => continue,
        };
        if offsets.is_some_and(|offsets| !offsets.contains(&offset)) {
            continue;
        }
        record_projected_call!();
        let mut projector = Projector {
            semantic,
            call_offset: offset,
            remaining: 256,
            exported: &exported,
            immutable: &mut immutable,
        };
        let mut spread = false;
        let spread_from = arguments
            .iter()
            .position(|argument| argument.as_expression().is_none());
        let values = arguments
            .iter()
            .take(32)
            .map(|argument| {
                let Some(expression) = argument.as_expression() else {
                    spread = true;
                    return ArgumentValue::Unknown;
                };
                if spread {
                    ArgumentValue::Unknown
                } else {
                    projector.value(expression, 0)
                }
            })
            .collect();
        result.insert(
            offset,
            ArgumentValues {
                values,
                spread_from,
            },
        );
    }
    if dynamic_scope {
        for call in result.values_mut() {
            call.values.fill(ArgumentValue::Unknown);
            call.spread_from = Some(0);
        }
    }
    result
}

include!("arguments/projector.rs");
