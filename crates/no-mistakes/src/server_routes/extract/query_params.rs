use oxc_ast::ast::{
    Argument, BindingPattern, Expression, FunctionBody, ObjectPropertyKind, Statement,
};
use std::collections::BTreeSet;

use super::ServerRouteVisitor;

impl ServerRouteVisitor<'_> {
    pub(super) fn query_params_from_call(
        &self,
        call: &oxc_ast::ast::CallExpression<'_>,
    ) -> Vec<String> {
        let mut params = BTreeSet::new();
        for arg in &call.arguments {
            self.collect_query_params_from_arg(arg, &mut params);
        }
        params.into_iter().collect()
    }

    pub(super) fn collect_query_params_from_arg(
        &self,
        arg: &Argument<'_>,
        params: &mut BTreeSet<String>,
    ) {
        if let Some(expr) = arg.as_expression() {
            self.collect_query_params_from_handler_expression(expr, params);
            if let Expression::Identifier(id) = expr {
                if let Some(handler_params) = self.handler_query_params.get(id.name.as_str()) {
                    params.extend(handler_params.iter().cloned());
                }
            }
        }
    }

    pub(super) fn collect_query_params_from_handler_expression(
        &self,
        expr: &Expression<'_>,
        params: &mut BTreeSet<String>,
    ) {
        match expr {
            Expression::ArrowFunctionExpression(arrow) => {
                for statement in &arrow.body.statements {
                    collect_query_params_from_statement(statement, params);
                }
            }
            Expression::FunctionExpression(function) => {
                collect_query_params_from_optional_function_body(function.body.as_ref(), params);
            }
            _ => {}
        }
    }
}

pub(super) fn collect_query_params_from_optional_function_body(
    body: Option<&oxc_allocator::Box<'_, FunctionBody<'_>>>,
    params: &mut BTreeSet<String>,
) {
    if let Some(body) = body {
        for statement in &body.statements {
            collect_query_params_from_statement(statement, params);
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
    let first = call.arguments.first()?;
    let Argument::StringLiteral(value) = first else {
        return None;
    };
    match property {
        "get" if !member_object_is_url_search_params(member.object()) => return None,
        "query" | "queries" if !member_object_is_request(member.object()) => return None,
        _ => {}
    }
    Some(value.value.as_str().to_string())
}

fn expression_is_query_object(expr: &Expression<'_>) -> bool {
    matches!(
        expr,
        Expression::StaticMemberExpression(member)
            if member.property.name == "query" && member_object_is_request(&member.object)
    )
}

fn member_object_is_request(expr: &Expression<'_>) -> bool {
    match expr {
        Expression::Identifier(id) => matches!(id.name.as_str(), "req" | "request"),
        Expression::StaticMemberExpression(member) => member.property.name == "req",
        _ => false,
    }
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
