//! Top-level callables whose body is one `return`, however they are declared.

use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{
    ArrowFunctionBody, BindingPattern, Declaration, ExportDefaultDeclarationKind, Expression,
    FormalParameters, Function, Program, Statement, VariableDeclaration, VariableDeclarationKind,
};

/// Top-level callables of `program` that return a bare expression, keyed by name.
pub(crate) fn collect<'a>(
    program: &'a Program<'a>,
) -> Vec<(&'a str, &'a FormalParameters<'a>, &'a Expression<'a>)> {
    let mut found = Vec::new();
    for statement in &program.body {
        match statement {
            Statement::FunctionDeclaration(function) => function_returns(function, &mut found),
            Statement::VariableDeclaration(declaration) => {
                variable_returns(declaration, &mut found)
            }
            Statement::ExportDefaultDeclaration(export) => {
                if let ExportDefaultDeclarationKind::FunctionDeclaration(function) =
                    &export.declaration
                {
                    function_returns(function, &mut found);
                }
            }
            Statement::ExportDeclaration(export) => match &export.declaration {
                Declaration::FunctionDeclaration(function) => {
                    function_returns(function, &mut found)
                }
                Declaration::VariableDeclaration(declaration) => {
                    variable_returns(declaration, &mut found);
                }
                _ => {}
            },
            _ => {}
        }
    }
    found
}

type Returns<'a> = Vec<(&'a str, &'a FormalParameters<'a>, &'a Expression<'a>)>;

fn function_returns<'a>(function: &'a Function<'a>, found: &mut Returns<'a>) {
    let (Some(id), Some(body)) = (&function.id, &function.body) else {
        return;
    };
    if function.r#async || function.generator {
        return;
    }
    if let Some(returned) = sole_return(&body.statements) {
        found.push((id.name.as_str(), &function.params, returned));
    }
}

fn variable_returns<'a>(declaration: &'a VariableDeclaration<'a>, found: &mut Returns<'a>) {
    if declaration.kind != VariableDeclarationKind::Const {
        return;
    }
    for declarator in &declaration.declarations {
        let (BindingPattern::BindingIdentifier(ident), Some(init)) =
            (&declarator.id, &declarator.init)
        else {
            continue;
        };
        match unwrap_ts_wrappers(init) {
            Expression::FunctionExpression(function)
                if !function.r#async
                    && !function.generator
                    && function
                        .id
                        .as_ref()
                        .is_none_or(|id| id.name.as_str() == ident.name.as_str()) =>
            {
                let returned = function
                    .body
                    .as_ref()
                    .and_then(|body| sole_return(&body.statements));
                if let Some(returned) = returned {
                    found.push((ident.name.as_str(), &function.params, returned));
                }
            }
            Expression::ArrowFunctionExpression(arrow) if !arrow.r#async => {
                let returned = match &arrow.body {
                    ArrowFunctionBody::FunctionBody(body) => sole_return(&body.statements),
                    other => other.as_expression(),
                };
                if let Some(returned) = returned {
                    found.push((ident.name.as_str(), &arrow.params, returned));
                }
            }
            _ => {}
        }
    }
}

fn sole_return<'a>(statements: &'a [Statement<'a>]) -> Option<&'a Expression<'a>> {
    match statements {
        [Statement::ReturnStatement(statement)] => statement.argument.as_ref(),
        _ => None,
    }
}
