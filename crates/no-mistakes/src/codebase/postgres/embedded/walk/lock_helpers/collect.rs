use super::{ExecutorAccess, Helper, LockHelpers};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{
    BindingPattern, Declaration, Expression, FormalParameters, FunctionBody, Program, Statement,
};

pub(in crate::codebase::postgres::embedded::walk) fn collect<'a>(
    program: &'a Program<'a>,
    enabled: bool,
) -> LockHelpers<'a> {
    let mut helpers = LockHelpers::default();
    if !enabled {
        return helpers;
    }
    let reassigned =
        super::super::resolve::functions::reassigned::ReassignedNames::collect(program);
    for statement in &program.body {
        let function = match statement {
            Statement::FunctionDeclaration(function) => Some(function.as_ref()),
            Statement::ExportDeclaration(export) => match &export.declaration {
                Declaration::FunctionDeclaration(function) => Some(function.as_ref()),
                _ => None,
            },
            _ => None,
        };
        let Some(function) = function.filter(|function| !function.generator) else {
            continue;
        };
        let Some(name) = function.id.as_ref().map(|id| id.name.as_str()) else {
            continue;
        };
        if reassigned.contains(name) {
            continue;
        }
        if let Some(helper) = function
            .body
            .as_ref()
            .and_then(|body| summarize(&function.params, body))
        {
            helpers.insert(name.to_owned(), helper);
        }
    }
    helpers
}
fn summarize<'a>(
    params: &'a FormalParameters<'a>,
    body: &'a FunctionBody<'a>,
) -> Option<Helper<'a>> {
    if params.rest.is_some() {
        return None;
    }
    let params = params
        .items
        .iter()
        .map(|param| match &param.pattern {
            BindingPattern::BindingIdentifier(id) if param.initializer.is_none() => {
                Some(id.name.as_str())
            }
            _ => None,
        })
        .collect::<Option<Vec<_>>>()?;
    let mut calls = Vec::new();
    for statement in &body.statements {
        let expression = match statement {
            Statement::ExpressionStatement(statement) => &statement.expression,
            Statement::ReturnStatement(statement) => statement.argument.as_ref()?,
            _ => return None,
        };
        let expression = match unwrap_ts_wrappers(expression) {
            Expression::AwaitExpression(awaited) => unwrap_ts_wrappers(&awaited.argument),
            expression if matches!(statement, Statement::ReturnStatement(_)) => expression,
            _ => return None,
        };
        let Expression::CallExpression(call) = expression else {
            return None;
        };
        if call.optional || call.arguments.len() > 2 {
            return None;
        }
        let access = if matches!(
            unwrap_ts_wrappers(&call.callee),
            Expression::StaticMemberExpression(_)
        ) {
            ExecutorAccess::Object
        } else {
            ExecutorAccess::Callable
        };
        let receiver = match unwrap_ts_wrappers(&call.callee) {
            Expression::StaticMemberExpression(member)
                if member.property.name == "query" && !member.optional =>
            {
                unwrap_ts_wrappers(&member.object)
            }
            receiver => receiver,
        };
        let Expression::Identifier(id) = receiver else {
            return None;
        };
        let executor = params.iter().position(|param| *param == id.name.as_str())?;
        let sql = unwrap_ts_wrappers(call.arguments.first()?.as_expression()?);
        if !matches!(sql, Expression::StringLiteral(_))
            && !matches!(sql, Expression::TemplateLiteral(template) if template.expressions.is_empty())
        {
            return None;
        }
        crate::codebase::postgres::embedded::sql_text(sql)?;
        if let Some(argument) = call.arguments.get(1) {
            let Expression::ArrayExpression(array) = unwrap_ts_wrappers(argument.as_expression()?)
            else {
                return None;
            };
            if array
                .elements
                .iter()
                .any(|element| element.as_expression().is_none())
            {
                return None;
            }
        }
        calls.push((executor, access, call.as_ref()));
        if matches!(statement, Statement::ReturnStatement(_)) {
            break;
        }
    }
    (!calls.is_empty()).then_some(Helper { params, calls })
}
