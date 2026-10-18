use super::ScopeVisitor;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{CallExpression, Expression};

impl ScopeVisitor<'_> {
    pub(in crate::codebase::postgres::embedded::walk) fn namespace_expression(
        &self,
        expression: &Expression<'_>,
    ) -> bool {
        match unwrap_ts_wrappers(expression) {
            Expression::Identifier(identifier) => {
                let name = identifier.name.as_str();
                match self
                    .scopes
                    .iter()
                    .enumerate()
                    .rev()
                    .find(|(_, scope)| scope.contains_key(name))
                {
                    Some((index, scope)) => {
                        scope[name].executor_alias == Some(true)
                            && self.namespace_proofs[index].contains(name)
                    }
                    None => self.imported_namespace_names.contains(name),
                }
            }
            Expression::CallExpression(call) => self.configured_require(call),
            Expression::ConditionalExpression(conditional) => {
                self.namespace_expression(&conditional.consequent)
                    || self.namespace_expression(&conditional.alternate)
            }
            _ => false,
        }
    }

    pub(in crate::codebase::postgres::embedded::walk) fn namespace_query_valid(&self) -> bool {
        self.query_binding && !self.namespace_query_invalidated
    }

    pub(in crate::codebase::postgres::embedded::walk) fn revoke_query_receiver(
        &mut self,
        receiver: &Expression<'_>,
    ) {
        if self.namespace_expression(receiver) {
            self.namespace_query_invalidated = true;
            for (scope, proofs) in self.scopes.iter_mut().zip(&self.namespace_proofs) {
                for (name, binding) in scope {
                    if proofs.contains(name) {
                        binding.executor_alias = None;
                    }
                }
            }
        }
        if let Expression::Identifier(identifier) = unwrap_ts_wrappers(receiver) {
            self.mark_dynamic(identifier.name.as_str());
        }
    }

    pub(in crate::codebase::postgres::embedded::walk) fn revoke_namespace_arguments(
        &mut self,
        call: &CallExpression<'_>,
    ) {
        for argument in &call.arguments {
            if let Some(expression) = argument.as_expression() {
                if self.namespace_expression(expression) {
                    self.revoke_query_receiver(expression);
                }
            }
        }
    }
}
