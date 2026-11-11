use crate::fx::FxHashMap;
use oxc_ast::{ast::*, AstKind};
use oxc_semantic::{NodeId, Semantic};

pub(super) fn calls_per_function(semantic: &Semantic<'_>) -> FxHashMap<NodeId, usize> {
    let mut calls = FxHashMap::default();
    for node in semantic.nodes().iter() {
        let effect = match node.kind() {
            AstKind::CallExpression(_) | AstKind::TaggedTemplateExpression(_) => true,
            AstKind::NewExpression(new) => {
                !matches!(new.callee.get_inner_expression(), Expression::Identifier(id) if id.name == "Error" && new.arguments.iter().all(|argument| matches!(argument, Argument::StringLiteral(_) | Argument::NumericLiteral(_) | Argument::BooleanLiteral(_) | Argument::NullLiteral(_))) && id.reference_id.get().is_some_and(|reference| semantic.scoping().get_reference(reference).symbol_id().is_none()))
            }
            _ => false,
        };
        if effect {
            if let Some(function) = semantic.nodes().ancestors(node.id()).find(|node| {
                matches!(
                    node.kind(),
                    AstKind::Function(_) | AstKind::ArrowFunctionExpression(_)
                )
            }) {
                *calls.entry(function.id()).or_default() += 1;
            }
        }
    }
    calls
}
