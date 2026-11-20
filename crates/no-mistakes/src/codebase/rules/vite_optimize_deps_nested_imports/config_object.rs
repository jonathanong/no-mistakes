//! Resolve the object literal a Vite config exports by default.

use oxc_ast::ast::{
    Argument, BindingPattern, ExportDefaultDeclarationKind, Expression, ObjectExpression, Program,
    Statement,
};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn default_object<'a>(program: &'a Program<'a>) -> Option<&'a ObjectExpression<'a>> {
    let bindings = top_level_bindings(program);
    for statement in &program.body {
        let Statement::ExportDefaultDeclaration(export) = statement else {
            continue;
        };
        let mut seen = BTreeSet::new();
        return export_object(&export.declaration, &bindings, &mut seen);
    }
    None
}

fn top_level_bindings<'a>(program: &'a Program<'a>) -> BTreeMap<String, &'a Expression<'a>> {
    let mut bindings = BTreeMap::new();
    for statement in &program.body {
        let Statement::VariableDeclaration(declaration) = statement else {
            continue;
        };
        for declarator in &declaration.declarations {
            let Some(name) = binding_name(&declarator.id) else {
                continue;
            };
            let Some(init) = declarator.init.as_ref() else {
                continue;
            };
            bindings.insert(name.to_string(), init);
        }
    }
    bindings
}

fn binding_name<'a>(binding: &'a BindingPattern<'a>) -> Option<&'a str> {
    match binding {
        BindingPattern::BindingIdentifier(identifier) => Some(identifier.name.as_str()),
        _ => None,
    }
}

fn export_object<'a>(
    export: &'a ExportDefaultDeclarationKind<'a>,
    bindings: &BTreeMap<String, &'a Expression<'a>>,
    seen: &mut BTreeSet<String>,
) -> Option<&'a ObjectExpression<'a>> {
    match export {
        ExportDefaultDeclarationKind::ObjectExpression(object) => Some(object),
        ExportDefaultDeclarationKind::CallExpression(call) => call
            .arguments
            .first()
            .and_then(|argument| argument_object(argument, bindings, seen)),
        ExportDefaultDeclarationKind::Identifier(identifier) => {
            identifier_object(identifier.name.as_str(), bindings, seen)
        }
        ExportDefaultDeclarationKind::ParenthesizedExpression(parenthesized) => {
            expression_object(&parenthesized.expression, bindings, seen)
        }
        _ => None,
    }
}

fn argument_object<'a>(
    argument: &'a Argument<'a>,
    bindings: &BTreeMap<String, &'a Expression<'a>>,
    seen: &mut BTreeSet<String>,
) -> Option<&'a ObjectExpression<'a>> {
    match argument {
        Argument::ObjectExpression(object) => Some(object),
        Argument::Identifier(identifier) => {
            identifier_object(identifier.name.as_str(), bindings, seen)
        }
        Argument::ParenthesizedExpression(parenthesized) => {
            expression_object(&parenthesized.expression, bindings, seen)
        }
        _ => None,
    }
}

fn expression_object<'a>(
    expression: &'a Expression<'a>,
    bindings: &BTreeMap<String, &'a Expression<'a>>,
    seen: &mut BTreeSet<String>,
) -> Option<&'a ObjectExpression<'a>> {
    match expression {
        Expression::ObjectExpression(object) => Some(object),
        Expression::CallExpression(call) => call
            .arguments
            .first()
            .and_then(|argument| argument_object(argument, bindings, seen)),
        Expression::Identifier(identifier) => {
            identifier_object(identifier.name.as_str(), bindings, seen)
        }
        Expression::ParenthesizedExpression(parenthesized) => {
            expression_object(&parenthesized.expression, bindings, seen)
        }
        _ => None,
    }
}

fn identifier_object<'a>(
    name: &str,
    bindings: &BTreeMap<String, &'a Expression<'a>>,
    seen: &mut BTreeSet<String>,
) -> Option<&'a ObjectExpression<'a>> {
    if !seen.insert(name.to_string()) {
        return None;
    }
    let object = bindings
        .get(name)
        .and_then(|expression| expression_object(expression, bindings, seen));
    seen.remove(name);
    object
}
