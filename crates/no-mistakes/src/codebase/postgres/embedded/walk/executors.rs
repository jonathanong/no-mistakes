//! Immutable executor aliases reuse the SQL walk's lexical binding ownership.
use super::ScopeVisitor;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{BindingPattern, Expression, VariableDeclarator};

impl ScopeVisitor<'_> {
    pub(super) fn relative_declaration_proof(&self, name: &str, start: u32) -> Option<u32> {
        self.provisional
            .iter()
            .position(|span| span.name == name && span.start == start)
            .map(|index| index as u32)
    }

    pub(super) fn relative_executor_proof(&self, expression: &Expression<'_>) -> Option<u32> {
        let Expression::Identifier(identifier) = unwrap_ts_wrappers(expression) else {
            return None;
        };
        let name = identifier.name.as_str();
        self.scopes
            .iter()
            .enumerate()
            .rev()
            .find(|(_, scope)| scope.contains_key(name))
            .and_then(|(index, scope)| {
                (scope[name].executor_alias == Some(false))
                    .then(|| self.relative_proofs[index].get(name).copied())
                    .flatten()
            })
    }

    pub(super) fn relative_executor_owners(&self, expression: &Expression<'_>) -> Vec<u32> {
        let Some(index) = self.relative_executor_proof(expression) else {
            return Vec::new();
        };
        self.provisional[index as usize].owners.clone()
    }

    pub(super) fn proven_executor(&self, expression: &Expression<'_>) -> bool {
        match unwrap_ts_wrappers(expression) {
            Expression::Identifier(identifier) => {
                let name = identifier.name.as_str();
                match self.scopes.iter().rev().find_map(|scope| scope.get(name)) {
                    Some(binding) => binding.executor_alias == Some(true),
                    None => {
                        self.bindings.contains(name) || self.imported_namespace_names.contains(name)
                    }
                }
            }
            Expression::ConditionalExpression(conditional) => {
                self.proven_executor(&conditional.consequent)
                    && self.proven_executor(&conditional.alternate)
            }
            Expression::CallExpression(call) => self.configured_require(call),
            _ => false,
        }
    }

    fn configured_require(&self, call: &oxc_ast::ast::CallExpression<'_>) -> bool {
        if self.configured_module.is_empty()
            || call.arguments.len() != 1
            || self
                .scopes
                .iter()
                .any(|scope| scope.contains_key("require"))
        {
            return false;
        }
        let Expression::Identifier(callee) = unwrap_ts_wrappers(&call.callee) else {
            return false;
        };
        let Some(Expression::StringLiteral(module)) =
            call.arguments.first().and_then(|arg| arg.as_expression())
        else {
            return false;
        };
        callee.name == "require"
            && super::super::scoped_bindings::from_configured_module(
                module.value.as_str(),
                &self.configured_module,
            )
    }

    pub(super) fn executor_alias_callee(&self, expression: &Expression<'_>) -> bool {
        match unwrap_ts_wrappers(expression) {
            Expression::Identifier(identifier) => {
                self.proven_executor(expression)
                    && !self.namespace_expression(expression)
                    && self
                        .scopes
                        .iter()
                        .rev()
                        .any(|scope| scope.contains_key(identifier.name.as_str()))
            }
            Expression::ConditionalExpression(_) => {
                self.proven_executor(expression) && !self.namespace_expression(expression)
            }
            _ => false,
        }
    }

    pub(super) fn initialize_executor_alias(
        &mut self,
        declaration: &VariableDeclarator<'_>,
        proven: bool,
        namespace: bool,
    ) {
        let BindingPattern::BindingIdentifier(identifier) = &declaration.id else {
            return;
        };
        let Some(binding) = self
            .scopes
            .last_mut()
            .and_then(|scope| scope.get_mut(identifier.name.as_str()))
        else {
            return;
        };
        // Predeclaration reserves lexical shadows; only the actual initializer
        // visit can promote an immutable binding, never a mutable declaration.
        if binding.executor_alias == Some(false) && proven {
            binding.executor_alias = Some(true);
            if namespace {
                self.namespace_proofs
                    .last_mut()
                    .expect("declaration scope")
                    .insert(identifier.name.to_string());
            }
        }
    }
}

mod namespace;
pub(super) use namespace::MemberQueryProof;

#[cfg(test)]
mod tests;
