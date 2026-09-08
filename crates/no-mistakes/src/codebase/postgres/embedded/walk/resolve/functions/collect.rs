use super::Resolvable;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{
    ArrowFunctionBody, BindingPattern, Declaration, Expression, Function, Statement,
    VariableDeclaration, VariableDeclarationKind, VariableDeclarator,
};
use std::collections::HashMap;

pub(super) fn collect_named_functions<'a>(
    statement: &'a Statement<'a>,
    raw: &mut HashMap<&'a str, Resolvable<'a>>,
) {
    match statement {
        Statement::FunctionDeclaration(function) => insert_function(function, raw),
        Statement::VariableDeclaration(declaration) => insert_const_functions(declaration, raw),
        Statement::ExportDeclaration(export) => match &export.declaration {
            Declaration::FunctionDeclaration(function) => insert_function(function, raw),
            Declaration::VariableDeclaration(declaration) => {
                insert_const_functions(declaration, raw);
            }
            _ => {}
        },
        _ => {}
    }
}

fn insert_function<'a>(function: &'a Function<'a>, raw: &mut HashMap<&'a str, Resolvable<'a>>) {
    if let Some((name, resolvable)) = function_resolvable(function) {
        raw.insert(name, resolvable);
    }
}

fn insert_const_functions<'a>(
    declaration: &'a VariableDeclaration<'a>,
    raw: &mut HashMap<&'a str, Resolvable<'a>>,
) {
    if declaration.kind != VariableDeclarationKind::Const {
        return;
    }
    for declarator in &declaration.declarations {
        if let Some((name, resolvable)) = const_resolvable(declarator) {
            raw.insert(name, resolvable);
        }
    }
}

fn function_resolvable<'a>(function: &'a Function<'a>) -> Option<(&'a str, Resolvable<'a>)> {
    let id = function.id.as_ref()?;
    let resolvable = resolvable_body(function)?;
    Some((id.name.as_str(), resolvable))
}

fn resolvable_body<'a>(function: &'a Function<'a>) -> Option<Resolvable<'a>> {
    if function.r#async || function.generator {
        return None;
    }
    let body = function.body.as_ref()?;
    Some(Resolvable {
        params: &function.params,
        body,
    })
}

/// A `const`-bound named function expression whose internal name differs
/// from the const's own binding is rejected outright rather than collected:
/// a reference to that internal name from inside the function's own body
/// resolves to the function expression's own self-binding at runtime, not to
/// whatever top-level declaration of the same name `raw` would otherwise
/// route it through — so resolving through `raw` there would report the
/// wrong function's body.
fn const_resolvable<'a>(
    declarator: &'a VariableDeclarator<'a>,
) -> Option<(&'a str, Resolvable<'a>)> {
    let BindingPattern::BindingIdentifier(ident) = &declarator.id else {
        return None;
    };
    let init = declarator.init.as_ref()?;
    match unwrap_ts_wrappers(init) {
        Expression::FunctionExpression(function) => {
            if function
                .id
                .as_ref()
                .is_some_and(|id| id.name.as_str() != ident.name.as_str())
            {
                return None;
            }
            let resolvable = resolvable_body(function)?;
            Some((ident.name.as_str(), resolvable))
        }
        Expression::ArrowFunctionExpression(arrow) => {
            if arrow.r#async {
                return None;
            }
            let ArrowFunctionBody::FunctionBody(body) = &arrow.body else {
                return None;
            };
            Some((
                ident.name.as_str(),
                Resolvable {
                    params: &arrow.params,
                    body,
                },
            ))
        }
        _ => None,
    }
}
