use super::ScopeVisitor;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{
    Argument, BindingPattern, CallExpression, Expression, FormalParameters, IdentifierReference,
    Statement,
};
use std::collections::HashSet;

impl ScopeVisitor<'_> {
    pub(super) fn note_preproof_reference(&mut self, id: &IdentifierReference<'_>) {
        let Some((index, binding)) = self
            .scopes
            .iter()
            .enumerate()
            .rev()
            .find_map(|(index, scope)| scope.get(id.name.as_str()).map(|binding| (index, binding)))
        else {
            return;
        };
        if binding.executor_alias == Some(false) {
            self.preproof_references.insert((
                self.member_query_proof.scope_ids[index],
                id.name.to_string(),
            ));
        }
    }

    pub(super) fn suppress_forwarded_body(
        &mut self,
        declaration: &oxc_ast::ast::VariableDeclarator<'_>,
        forwarded: Option<u32>,
    ) {
        let (Some(span), BindingPattern::BindingIdentifier(id)) = (forwarded, &declaration.id)
        else {
            return;
        };
        let scope = *self
            .member_query_proof
            .scope_ids
            .last()
            .expect("declaration scope");
        if !self
            .preproof_references
            .contains(&(scope, id.name.to_string()))
        {
            self.forwarded_calls.insert(span);
        }
    }

    pub(super) fn forwarded_executor_wrapper(&self, init: &Expression<'_>) -> Option<u32> {
        let (params, call, self_name) = match unwrap_ts_wrappers(init) {
            Expression::ArrowFunctionExpression(arrow) => {
                let body = arrow.body.as_expression().or_else(|| {
                    arrow
                        .body
                        .as_function_body()
                        .and_then(|body| sole_call(&body.statements))
                })?;
                let Expression::CallExpression(call) = unwrap_ts_wrappers(body) else {
                    return None;
                };
                (&arrow.params, call, None)
            }
            Expression::FunctionExpression(function) if !function.generator => {
                let body = function.body.as_ref()?;
                let call = sole_call(&body.statements)?;
                let Expression::CallExpression(call) = unwrap_ts_wrappers(call) else {
                    return None;
                };
                (
                    &function.params,
                    call,
                    function.id.as_ref().map(|id| id.name.as_str()),
                )
            }
            _ => return None,
        };
        let Expression::Identifier(callee) = unwrap_ts_wrappers(&call.callee) else {
            return None;
        };
        if call.optional
            || self_name == Some(callee.name.as_str())
            || !self.proven_executor(&call.callee)
        {
            return None;
        }
        if !forwards(params, call, callee.name.as_str()) {
            return None;
        }
        Some(call.span.start)
    }
}

fn sole_call<'a, 'b>(statements: &'b [Statement<'a>]) -> Option<&'b Expression<'a>> {
    match statements {
        [Statement::ReturnStatement(statement)] => statement.argument.as_ref(),
        [Statement::ExpressionStatement(statement)] => Some(&statement.expression),
        _ => None,
    }
}

fn forwards(params: &FormalParameters<'_>, call: &CallExpression<'_>, callee: &str) -> bool {
    if params.rest.is_some()
        || call.arguments.is_empty()
        || call.arguments.len() > params.items.len()
        || (call.arguments.len() < params.items.len() && call.arguments.len() < 2)
    {
        return false;
    }
    let mut names = HashSet::new();
    params.items.iter().enumerate().all(|(index, param)| {
        if param.initializer.is_some() || !param.decorators.is_empty() {
            return false;
        }
        let BindingPattern::BindingIdentifier(binding) = &param.pattern else {
            return false;
        };
        if binding.name == callee || !names.insert(binding.name.as_str()) {
            return false;
        }
        if index >= call.arguments.len() {
            return true;
        }
        let Argument::Identifier(argument) = &call.arguments[index] else {
            return false;
        };
        argument.name == binding.name
    })
}
