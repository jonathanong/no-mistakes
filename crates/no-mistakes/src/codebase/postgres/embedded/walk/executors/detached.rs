use super::super::ScopeVisitor;
use crate::codebase::postgres::executor_detachment::{bind_callee, bind_method, query_receiver};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{Argument, BindingPattern, CallExpression, Expression, VariableDeclarator};

impl ScopeVisitor<'_> {
    pub(in crate::codebase::postgres::embedded::walk) fn detached_query_receiver(
        &self,
        receiver: &Expression<'_>,
    ) -> bool {
        if !self.query_binding {
            return false;
        }
        let origins = self.receiver_origins(receiver);
        self.namespace_expression(receiver)
            || (self.proven_executor(receiver)
                && origins.iter().all(|origin| self.detached_origin(origin)))
            || (!self.configured_module.is_empty()
                && origins.iter().any(|origin| {
                    !matches!(origin, super::namespace::Origin::Binding(_, _))
                        && self.detached_origin(origin)
                }))
    }

    pub(in crate::codebase::postgres::embedded::walk) fn detached_receiver_proven(
        &self,
        receiver: &Expression<'_>,
    ) -> bool {
        if !self.detached_query_receiver(receiver) {
            return false;
        }
        if self.proven_executor(receiver) {
            return true;
        }
        match unwrap_ts_wrappers(receiver) {
            Expression::Identifier(name) => {
                if self
                    .lookup(name.name.as_str())
                    .is_some_and(|binding| binding.executor_alias.is_none())
                {
                    return false;
                }
            }
            Expression::StaticMemberExpression(member) => {
                return self.detached_receiver_proven(&member.object)
            }
            Expression::ComputedMemberExpression(member) => {
                return self.detached_receiver_proven(&member.object)
            }
            Expression::ConditionalExpression(branch) => {
                return self.detached_receiver_proven(&branch.consequent)
                    && self.detached_receiver_proven(&branch.alternate)
            }
            _ => return false,
        }
        let origins = self.receiver_origins(receiver);
        !origins.is_empty() && origins.iter().all(|origin| self.detached_origin(origin))
    }

    fn detached_origin(&self, origin: &super::namespace::Origin) -> bool {
        match origin {
            super::namespace::Origin::Module(module) => module == &self.configured_module,
            super::namespace::Origin::Export(module, name) => {
                module == &self.configured_module && !self.executor_options.is_callable_export(name)
            }
            super::namespace::Origin::Binding(_, _) => true,
        }
    }

    pub(in crate::codebase::postgres::embedded::walk) fn uncertain_detached_member(
        &self,
        receiver: &Expression<'_>,
    ) -> bool {
        self.has_query_capture(receiver)
            && self.detached_query_receiver(receiver)
            && self.receiver_origins(receiver).iter().any(|origin| {
                !matches!(origin, super::namespace::Origin::Binding(_, _))
                    && self.detached_origin(origin)
            })
    }

    pub(in crate::codebase::postgres::embedded::walk) fn valid_query_bind(
        &self,
        callee: &Expression<'_>,
        arguments: &[Argument<'_>],
    ) -> bool {
        let Some(method) = bind_callee(callee) else {
            return false;
        };
        let origins = if let Some(receiver) =
            query_receiver(method).filter(|receiver| self.detached_receiver_proven(receiver))
        {
            self.receiver_origins(receiver)
        } else {
            self.query_capture_origins(method).unwrap_or_default()
        };
        arguments.len() == 1
            && !origins.is_empty()
            && arguments[0]
                .as_expression()
                .is_some_and(|receiver| self.receiver_origins(receiver) == origins)
    }

    pub(in crate::codebase::postgres::embedded::walk) fn initialize_detached_query(
        &mut self,
        declaration: &VariableDeclarator<'_>,
        constant: bool,
    ) {
        let (BindingPattern::BindingIdentifier(local), Some(init)) =
            (&declaration.id, &declaration.init)
        else {
            return;
        };
        self.initialize_detached_value(local.name.as_str(), init, constant);
    }

    pub(in crate::codebase::postgres::embedded::walk) fn initialize_detached_value(
        &mut self,
        name: &str,
        init: &Expression<'_>,
        constant: bool,
    ) {
        if self.copy_query_capture(name, init, constant) {
            return;
        }
        if let Some(receiver) =
            query_receiver(init).filter(|receiver| self.detached_query_receiver(receiver))
        {
            self.note_query_capture_with_certainty(
                name,
                receiver,
                constant && self.detached_receiver_proven(receiver),
            );
            return;
        }
        let Expression::CallExpression(call) = unwrap_ts_wrappers(init) else {
            return;
        };
        self.initialize_bound_query(name, call, constant);
    }

    fn initialize_bound_query(&mut self, name: &str, call: &CallExpression<'_>, constant: bool) {
        let Some(method) = bind_method(call) else {
            return;
        };
        let trusted = constant && self.valid_query_bind(&call.callee, &call.arguments);
        if self.copy_query_capture(name, method, trusted) {
            return;
        }
        if let Some(receiver) =
            query_receiver(method).filter(|receiver| self.detached_query_receiver(receiver))
        {
            self.note_query_capture_with_certainty(name, receiver, trusted);
        }
    }
}
