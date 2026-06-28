use oxc_ast::ast::{
    Argument, BindingPattern, ChainElement, Expression, FunctionBody, ObjectPropertyKind, Statement,
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

    pub(super) fn query_params_from_function_body(
        &self,
        body: &'_ [Statement<'_>],
        named_handlers: &HashMap<String, BTreeSet<String>>,
    ) -> BTreeSet<String> {
        let mut params = BTreeSet::new();
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
                for statement in &arrow.body.statements {
                    collect_query_params_from_statement(statement, params, named_handlers);
                }
            }
            Expression::FunctionExpression(function) => {
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

include!("query_params_statements.rs");
include!("query_params_expressions.rs");

fn query_param_from_call(call: &oxc_ast::ast::CallExpression<'_>) -> Option<String> {
    let member = call.callee.as_member_expression()?;
    let property = member.static_property_name()?;
    if !matches!(property, "query" | "queries" | "get") {
        return None;
    }
    if matches!(property, "query" | "queries") && !is_query_call_object(member.object()) {
        return None;
    }
    let first = call.arguments.first()?;
    let Argument::StringLiteral(value) = first else {
        return None;
    };
    if property == "get" && !member_object_is_url_search_params(member.object()) {
        return None;
    }
    Some(value.value.as_str().to_string())
}

fn expression_is_query_object(expr: &Expression<'_>) -> bool {
    matches!(
        expr,
        Expression::StaticMemberExpression(member) if member.property.name == "query"
    )
}

fn is_query_call_object(expr: &Expression<'_>) -> bool {
    is_request_query_object(expr)
}

fn is_request_query_object(expr: &Expression<'_>) -> bool {
    match expr {
        Expression::ChainExpression(chain) => {
            is_request_query_object_from_chain_element(&chain.expression)
        }
        Expression::Identifier(id) => is_request_identifier(&id.name),
        Expression::StaticMemberExpression(member) => is_request_object_member(member, 0),
        _ => false,
    }
}

fn is_request_query_object_from_chain_element(chain: &ChainElement<'_>) -> bool {
    match chain {
        ChainElement::CallExpression(call) => is_request_query_object(&call.callee),
        other => other
            .as_member_expression()
            .is_some_and(is_request_object_member_expr),
    }
}

fn is_request_object_member_expr(member: &oxc_ast::ast::MemberExpression<'_>) -> bool {
    let Some(property) = member.static_property_name() else {
        return false;
    };
    if !matches!(property, "req" | "request" | "ctx" | "context" | "c") {
        return false;
    }
    is_request_object_expr(member.object(), 1)
}

fn is_request_object_expr(expr: &Expression<'_>, nesting: u8) -> bool {
    if nesting > 1 {
        return false;
    }
    match expr {
        Expression::Identifier(id) => is_request_identifier(&id.name),
        Expression::StaticMemberExpression(member) => is_request_object_member(member, nesting),
        _ => false,
    }
}

fn is_request_object_member(
    member: &oxc_ast::ast::StaticMemberExpression<'_>,
    nesting: u8,
) -> bool {
    if nesting > 1 {
        return false;
    }
    if !matches!(
        member.property.name.as_str(),
        "req" | "request" | "ctx" | "context" | "c"
    ) {
        return false;
    }
    match &member.object {
        Expression::Identifier(id) => is_request_identifier(&id.name),
        Expression::StaticMemberExpression(member) => is_request_object_member(member, nesting + 1),
        _ => false,
    }
}

fn is_request_identifier(name: &str) -> bool {
    matches!(name, "req" | "request" | "ctx" | "context" | "c")
}

fn member_object_is_url_search_params(expr: &Expression<'_>) -> bool {
    matches!(
        expr,
        Expression::NewExpression(new_expr)
            if matches!(&new_expr.callee, Expression::Identifier(id) if id.name == "URLSearchParams")
    )
}

fn collect_query_object_destructure_names(
    pattern: &BindingPattern<'_>,
    params: &mut BTreeSet<String>,
) {
    match pattern {
        BindingPattern::BindingIdentifier(_) => {}
        BindingPattern::ObjectPattern(object) => {
            for property in &object.properties {
                if let Some(name) = property.key.static_name() {
                    params.insert(name.to_string());
                } else {
                    collect_query_object_destructure_names(&property.value, params);
                }
            }
        }
        BindingPattern::AssignmentPattern(assign) => {
            collect_query_object_destructure_names(&assign.left, params);
        }
        BindingPattern::ArrayPattern(_) => {}
    }
}

#[cfg(test)]
#[path = "query_params_tests.rs"]
mod query_params_tests;
