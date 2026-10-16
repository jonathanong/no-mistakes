//! Immutable executor aliases reuse the SQL walk's lexical binding ownership.
use super::ScopeVisitor;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{BindingPattern, Expression, VariableDeclarator};

impl ScopeVisitor<'_> {
    pub(super) fn proven_executor(&self, expression: &Expression<'_>) -> bool {
        match unwrap_ts_wrappers(expression) {
            Expression::Identifier(identifier) => {
                let name = identifier.name.as_str();
                match self.scopes.iter().rev().find_map(|scope| scope.get(name)) {
                    Some(binding) => binding.executor_alias == Some(true),
                    None => self.bindings.contains(name),
                }
            }
            Expression::ConditionalExpression(conditional) => {
                self.proven_executor(&conditional.consequent)
                    && self.proven_executor(&conditional.alternate)
            }
            _ => false,
        }
    }

    pub(super) fn executor_alias_callee(&self, expression: &Expression<'_>) -> bool {
        match unwrap_ts_wrappers(expression) {
            Expression::Identifier(identifier) => self
                .scopes
                .iter()
                .rev()
                .find_map(|scope| scope.get(identifier.name.as_str()))
                .is_some_and(|binding| binding.executor_alias == Some(true)),
            Expression::ConditionalExpression(_) => self.proven_executor(expression),
            _ => false,
        }
    }

    pub(super) fn initialize_executor_alias(
        &mut self,
        declaration: &VariableDeclarator<'_>,
        proven: bool,
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
        if binding.executor_alias == Some(false) {
            binding.executor_alias = Some(proven);
        }
    }
}

#[cfg(test)]
mod tests;
