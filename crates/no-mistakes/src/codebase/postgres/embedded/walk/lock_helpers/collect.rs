use super::{Helper, LockHelpers};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{
    BindingPattern, Declaration, Expression, FormalParameters, FunctionBody, Program, Statement,
};

pub(in crate::codebase::postgres::embedded::walk) fn collect<'a>(
    program: &'a Program<'a>,
    enabled: bool,
    source: &str,
) -> LockHelpers {
    let mut helpers = LockHelpers::default();
    if !enabled {
        return helpers;
    }
    let reassigned =
        super::super::resolve::functions::reassigned::ReassignedNames::collect(program);
    for _ in 0..program.body.len() {
        let mut added = false;
        for statement in &program.body {
            let function = match statement {
                Statement::FunctionDeclaration(function) => Some(function.as_ref()),
                Statement::ExportDeclaration(export) => match &export.declaration {
                    Declaration::FunctionDeclaration(function) => Some(function.as_ref()),
                    _ => None,
                },
                _ => None,
            };
            let Some((function, name)) = function
                .filter(|function| !function.generator)
                .and_then(|function| function.id.as_ref().map(|id| (function, id.name.as_str())))
            else {
                continue;
            };
            if reassigned.contains(name) || helpers.contains_key(name) {
                continue;
            }
            if let Some(helper) = function
                .body
                .as_ref()
                .and_then(|body| summarize(&function.params, body, &helpers, source))
            {
                helpers.insert(name.to_owned(), helper);
                added = true;
            }
        }
        if !added {
            break;
        }
    }
    helpers
}
fn summarize<'a>(
    params: &'a FormalParameters<'a>,
    body: &'a FunctionBody<'a>,
    helpers: &LockHelpers,
    source: &str,
) -> Option<Helper> {
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
        if call.optional
            || call
                .arguments
                .iter()
                .any(|argument| argument.as_expression().is_none())
        {
            return None;
        }
        if let Expression::Identifier(id) = unwrap_ts_wrappers(&call.callee) {
            if !params.contains(&id.name.as_str()) {
                let helper = helpers.get(id.name.as_str())?;
                let projected = super::parameters::compose(helper, call, &params)?;
                if calls.len() + projected.len() > 256 {
                    return None;
                }
                calls.extend(projected);
                if matches!(statement, Statement::ReturnStatement(_)) {
                    break;
                }
                continue;
            }
        }
        if calls.len() >= 256 {
            return None;
        }
        calls.push(super::query::summarize(call, &params, source)?);
        if matches!(statement, Statement::ReturnStatement(_)) {
            break;
        }
    }
    (!calls.is_empty()).then_some(Helper { calls })
}
