use oxc_ast::ast::{
    Argument, BindingPattern, ChainElement, Expression, FormalParameters, FunctionBody,
    ObjectPropertyKind, Statement,
};
use std::collections::{BTreeSet, HashMap};

use super::ServerRouteVisitor;

impl ServerRouteVisitor<'_> {
    pub(super) fn query_params_from_call(
        &self,
        call: &oxc_ast::ast::CallExpression<'_>,
        named_handlers: &HashMap<String, BTreeSet<String>>,
    ) -> Vec<String> {
        let mut params = BTreeSet::new();
        for arg in &call.arguments {
            collect_query_params_from_arg(arg, &mut params, named_handlers);
        }
        params.into_iter().collect()
    }

    pub(super) fn query_params_from_function(
        &self,
        parameters: &FormalParameters<'_>,
        body: &'_ [Statement<'_>],
        named_handlers: &HashMap<String, BTreeSet<String>>,
    ) -> BTreeSet<String> {
        let mut params = BTreeSet::new();
        collect_query_params_from_formal_parameters(parameters, &mut params);
        for statement in body {
            collect_query_params_from_statement(statement, &mut params, named_handlers);
        }
        params
    }
}

fn collect_query_params_from_arg(
    arg: &Argument<'_>,
    params: &mut BTreeSet<String>,
    named_handlers: &HashMap<String, BTreeSet<String>>,
) {
    if let Argument::Identifier(id) = arg {
        if let Some(handler_params) = named_handlers.get(id.name.as_str()) {
            params.extend(handler_params.iter().cloned());
        }
        return;
    }
    if let Some(expr) = arg.as_expression() {
        match expr {
            Expression::ArrowFunctionExpression(arrow) => {
                collect_query_params_from_formal_parameters(&arrow.params, params);
                for statement in &arrow.body.statements {
                    collect_query_params_from_statement(statement, params, named_handlers);
                }
            }
            Expression::FunctionExpression(function) => {
                collect_query_params_from_formal_parameters(&function.params, params);
                collect_query_params_from_optional_function_body(
                    function.body.as_ref(),
                    params,
                    named_handlers,
                );
            }
            _ => {}
        }
    }
}

fn collect_query_params_from_optional_function_body(
    body: Option<&oxc_allocator::Box<'_, FunctionBody<'_>>>,
    params: &mut BTreeSet<String>,
    named_handlers: &HashMap<String, BTreeSet<String>>,
) {
    if let Some(body) = body {
        for statement in &body.statements {
            collect_query_params_from_statement(statement, params, named_handlers);
        }
    }
}

fn collect_query_params_from_formal_parameters(
    parameters: &FormalParameters<'_>,
    params: &mut BTreeSet<String>,
) {
    for parameter in &parameters.items {
        collect_query_params_from_parameter_pattern(&parameter.pattern, params);
    }
    if let Some(rest) = &parameters.rest {
        collect_query_params_from_parameter_pattern(&rest.rest.argument, params);
    }
}

fn collect_query_params_from_parameter_pattern(
    pattern: &BindingPattern<'_>,
    params: &mut BTreeSet<String>,
) {
    match pattern {
        BindingPattern::ObjectPattern(object) => {
            for property in &object.properties {
                if property
                    .key
                    .static_name()
                    .is_some_and(|name| name == "query")
                {
                    collect_query_object_destructure_names(&property.value, params);
                } else {
                    collect_query_params_from_parameter_pattern(&property.value, params);
                }
            }
        }
        BindingPattern::AssignmentPattern(_)
        | BindingPattern::BindingIdentifier(_)
        | BindingPattern::ArrayPattern(_) => {}
    }
}

include!("query_params_statements.rs");
include!("query_params_expressions.rs");
include!("query_params_objects.rs");

#[cfg(test)]
#[path = "query_params_tests.rs"]
mod query_params_tests;

#[cfg(test)]
#[path = "query_params_object_tests.rs"]
mod query_params_object_tests;
