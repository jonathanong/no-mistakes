//! Potential Node loader identities, never proof that a mutable call resolves.
use super::TypeScriptModuleFacts;
use crate::fx::FxHashMap;
use oxc_ast::{ast::*, AstKind};
use oxc_semantic::Semantic;
use oxc_syntax::symbol::SymbolId;

mod bindings;
mod origins;
#[cfg(test)]
mod tests;

#[derive(Debug, Clone)]
pub(crate) struct NodeLoaderCall {
    pub callee: String,
    pub specifier: String,
    pub computed: bool,
}

#[derive(Clone, Copy)]
enum Origin {
    GlobalRequire,
    ModuleObject,
    ModuleNamespace,
    Factory,
    Loader,
}

pub(super) struct Collector<'s, 'a> {
    semantic: &'s Semantic<'a>,
    bindings: FxHashMap<SymbolId, Option<Origin>>,
}

impl<'s, 'a> Collector<'s, 'a> {
    pub(super) fn new(semantic: &'s Semantic<'a>) -> Self {
        Self {
            semantic,
            bindings: FxHashMap::default(),
        }
    }

    pub(super) fn collect(
        &mut self,
        call: &'s CallExpression<'a>,
        facts: &mut TypeScriptModuleFacts,
    ) {
        if !matches!(self.origin(&call.callee), Some(Origin::Loader)) {
            return;
        }
        let Some(argument) = call.arguments.first() else {
            return;
        };
        let literal = argument
            .as_expression()
            .and_then(crate::codebase::dependencies::extract::static_import_specifier);
        let computed = literal.is_none();
        let specifier = literal.unwrap_or_else(|| match argument.as_expression() {
            Some(Expression::Identifier(id)) => id.name.to_string(),
            _ => "<computed>".into(),
        });
        facts
            .node_loader_calls
            .entry(call.span.start)
            .or_default()
            .push(NodeLoaderCall {
                callee: label(&call.callee),
                specifier,
                computed,
            });
    }
}

fn is_module(name: &str) -> bool {
    matches!(name, "module" | "node:module")
}

fn member<'e, 'a>(expression: &'e Expression<'a>) -> Option<(&'e Expression<'a>, &'e str)> {
    match expression {
        Expression::StaticMemberExpression(member) => {
            Some((&member.object, member.property.name.as_str()))
        }
        Expression::ComputedMemberExpression(member) => {
            let Expression::StringLiteral(name) = member.expression.get_inner_expression() else {
                return None;
            };
            Some((&member.object, name.value.as_str()?))
        }
        _ => None,
    }
}

fn label(mut expression: &Expression<'_>) -> String {
    let mut suffixes = Vec::new();
    let mut label = loop {
        match expression.get_inner_expression() {
            Expression::Identifier(id) => break id.name.to_string(),
            Expression::CallExpression(call) => {
                suffixes.push("(...)".into());
                expression = &call.callee;
            }
            member_expression => {
                let Some((object, property)) = member(member_expression) else {
                    break "<Node loader>".into();
                };
                suffixes.push(
                    if matches!(member_expression, Expression::ComputedMemberExpression(_)) {
                        format!("[{property:?}]")
                    } else {
                        format!(".{property}")
                    },
                );
                expression = object;
            }
        }
    };
    for suffix in suffixes.into_iter().rev() {
        label.push_str(&suffix);
    }
    label
}
