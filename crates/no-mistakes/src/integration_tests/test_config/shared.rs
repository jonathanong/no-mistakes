use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{BindingPattern, Declaration, Expression, FunctionBody, Program, Statement};
use std::collections::{BTreeMap, BTreeSet};

mod objects;

pub(in crate::integration_tests) use super::shared_literals::{
    inferred_string_or_array, optional_string, property_key_name, required_string,
};
pub(in crate::integration_tests) use objects::{
    default_export_object, expression_config_object, property_expression_deep,
};

pub(in crate::integration_tests) fn top_level_object_bindings<'a>(
    program: &'a Program<'a>,
) -> BTreeMap<String, &'a Expression<'a>> {
    let mut bindings = BTreeMap::new();
    for statement in &program.body {
        let declaration = match statement {
            Statement::VariableDeclaration(declaration) => Some(declaration),
            Statement::ExportNamedDeclaration(export) => match export.declaration.as_ref() {
                Some(Declaration::VariableDeclaration(declaration)) => Some(declaration),
                _ => None,
            },
            _ => None,
        };
        let Some(declaration) = declaration else {
            continue;
        };
        for declarator in &declaration.declarations {
            let (Some(name), Some(init)) =
                (binding_identifier_name(&declarator.id), &declarator.init)
            else {
                continue;
            };
            bindings.insert(name.to_string(), init);
        }
    }
    bindings
}

pub(in crate::integration_tests) fn function_body_bindings<'a>(
    body: &'a FunctionBody<'a>,
) -> BTreeMap<String, &'a Expression<'a>> {
    let mut bindings = BTreeMap::new();
    for statement in &body.statements {
        let Statement::VariableDeclaration(declaration) = statement else {
            continue;
        };
        for declarator in &declaration.declarations {
            let (Some(name), Some(init)) =
                (binding_identifier_name(&declarator.id), &declarator.init)
            else {
                continue;
            };
            bindings.insert(name.to_string(), init);
        }
    }
    bindings
}

pub(in crate::integration_tests) fn is_array_expression_reference(
    expression: &Expression<'_>,
    bindings: &BTreeMap<String, &Expression<'_>>,
) -> bool {
    if matches!(
        unwrap_ts_wrappers(expression),
        Expression::ArrayExpression(_)
    ) {
        return true;
    }
    let Expression::Identifier(identifier) = unwrap_ts_wrappers(expression) else {
        return false;
    };
    bindings
        .get(identifier.name.as_str())
        .is_some_and(|binding| {
            matches!(unwrap_ts_wrappers(binding), Expression::ArrayExpression(_))
        })
}

pub(in crate::integration_tests) fn expression_value<'a>(
    expression: &'a Expression<'a>,
    bindings: &BTreeMap<String, &'a Expression<'a>>,
) -> &'a Expression<'a> {
    let mut seen = BTreeSet::new();
    expression_value_inner(expression, bindings, &mut seen)
}

fn expression_value_inner<'a>(
    expression: &'a Expression<'a>,
    bindings: &BTreeMap<String, &'a Expression<'a>>,
    seen: &mut BTreeSet<String>,
) -> &'a Expression<'a> {
    let Expression::Identifier(identifier) = unwrap_ts_wrappers(expression) else {
        return expression;
    };
    if !seen.insert(identifier.name.to_string()) {
        return expression;
    }
    let resolved = bindings
        .get(identifier.name.as_str())
        .map_or(expression, |value| {
            expression_value_inner(value, bindings, seen)
        });
    seen.remove(identifier.name.as_str());
    resolved
}

fn binding_identifier_name<'a>(binding: &'a BindingPattern<'a>) -> Option<&'a str> {
    match binding {
        BindingPattern::BindingIdentifier(identifier) => Some(identifier.name.as_str()),
        _ => None,
    }
}
