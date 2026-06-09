use crate::playwright::ast;
use oxc_ast::ast::{
    Argument, AssignmentTarget, BindingPattern, ExportDefaultDeclarationKind, Expression,
    ObjectExpression, ObjectPropertyKind, Program, PropertyKey, Statement,
};
use std::collections::{BTreeMap, BTreeSet};

pub fn default_export_object<'a>(program: &'a Program<'a>) -> Option<&'a ObjectExpression<'a>> {
    let bindings = top_level_object_bindings(program);

    for statement in &program.body {
        if let Statement::ExportDefaultDeclaration(export) = statement {
            return export_config_object(&export.declaration, &bindings);
        }

        if let Some(object) = commonjs_config_object(statement, &bindings) {
            return Some(object);
        }
    }
    None
}

pub fn top_level_object_bindings<'a>(
    program: &'a Program<'a>,
) -> BTreeMap<String, &'a Expression<'a>> {
    let mut bindings = BTreeMap::new();
    for statement in &program.body {
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

pub fn binding_identifier_name<'a>(binding: &'a BindingPattern<'a>) -> Option<&'a str> {
    match binding {
        BindingPattern::BindingIdentifier(identifier) => Some(identifier.name.as_str()),
        _ => None,
    }
}

fn export_config_object<'a>(
    export: &'a ExportDefaultDeclarationKind<'a>,
    bindings: &BTreeMap<String, &'a Expression<'a>>,
) -> Option<&'a ObjectExpression<'a>> {
    match export {
        ExportDefaultDeclarationKind::ObjectExpression(object) => Some(object),
        ExportDefaultDeclarationKind::CallExpression(call) => {
            call.arguments.first().and_then(|argument| {
                let mut seen = BTreeSet::new();
                argument_config_object(argument, bindings, &mut seen)
            })
        }
        ExportDefaultDeclarationKind::Identifier(identifier) => {
            let mut seen = BTreeSet::new();
            identifier_config_object(identifier.name.as_str(), bindings, &mut seen)
        }
        ExportDefaultDeclarationKind::ParenthesizedExpression(parenthesized) => {
            let mut seen = BTreeSet::new();
            expression_config_object(&parenthesized.expression, bindings, &mut seen)
        }
        _ => None,
    }
}

fn commonjs_config_object<'a>(
    statement: &'a Statement<'a>,
    bindings: &BTreeMap<String, &'a Expression<'a>>,
) -> Option<&'a ObjectExpression<'a>> {
    let Statement::ExpressionStatement(statement) = statement else {
        return None;
    };
    let Expression::AssignmentExpression(assignment) = &statement.expression else {
        return None;
    };
    if assignment_target_path(&assignment.left)
        .as_deref()
        .is_none_or(|parts| parts != ["module", "exports"])
    {
        return None;
    }
    let mut seen = BTreeSet::new();
    expression_config_object(&assignment.right, bindings, &mut seen)
}

fn assignment_target_path(target: &AssignmentTarget<'_>) -> Option<Vec<String>> {
    match target {
        AssignmentTarget::StaticMemberExpression(member) => {
            let mut parts = ast::expression_path(&member.object)?;
            parts.push(member.property.name.to_string());
            Some(parts)
        }
        _ => None,
    }
}

fn argument_config_object<'a>(
    argument: &'a Argument<'a>,
    bindings: &BTreeMap<String, &'a Expression<'a>>,
    seen: &mut BTreeSet<String>,
) -> Option<&'a ObjectExpression<'a>> {
    match argument {
        Argument::ObjectExpression(object) => Some(object),
        Argument::Identifier(identifier) => {
            identifier_config_object(identifier.name.as_str(), bindings, seen)
        }
        Argument::ParenthesizedExpression(parenthesized) => {
            expression_config_object(&parenthesized.expression, bindings, seen)
        }
        // Unwrap nested wrapper calls such as
        // `defineConfig(createPlaywrightConfig({ ... }))`, where the inner helper
        // receives the config object literal as its first argument. Options added
        // inside the helper body are still invisible to static parsing (which is
        // why `tests.playwright.testIdAttribute` exists), but the literal that is
        // passed through is recovered.
        Argument::CallExpression(call) => call
            .arguments
            .first()
            .and_then(|argument| argument_config_object(argument, bindings, seen)),
        _ => None,
    }
}

pub fn expression_config_object<'a>(
    expression: &'a Expression<'a>,
    bindings: &BTreeMap<String, &'a Expression<'a>>,
    seen: &mut BTreeSet<String>,
) -> Option<&'a ObjectExpression<'a>> {
    match expression {
        Expression::ObjectExpression(object) => Some(object),
        Expression::Identifier(identifier) => {
            identifier_config_object(identifier.name.as_str(), bindings, seen)
        }
        Expression::CallExpression(call) => call
            .arguments
            .first()
            .and_then(|argument| argument_config_object(argument, bindings, seen)),
        Expression::ParenthesizedExpression(parenthesized) => {
            expression_config_object(&parenthesized.expression, bindings, seen)
        }
        _ => None,
    }
}

fn identifier_config_object<'a>(
    name: &str,
    bindings: &BTreeMap<String, &'a Expression<'a>>,
    seen: &mut BTreeSet<String>,
) -> Option<&'a ObjectExpression<'a>> {
    if !seen.insert(name.to_string()) {
        return None;
    }
    let object = bindings
        .get(name)
        .and_then(|expression| expression_config_object(expression, bindings, seen));
    seen.remove(name);
    object
}

pub fn property_expression<'a>(
    object: &'a ObjectExpression<'a>,
    name: &str,
) -> Option<&'a Expression<'a>> {
    for property in &object.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            continue;
        };
        if property.computed || property.method {
            continue;
        }
        if property_key_name(&property.key).as_deref() == Some(name) {
            return Some(&property.value);
        }
    }
    None
}

fn property_key_name(key: &PropertyKey<'_>) -> Option<String> {
    match key {
        PropertyKey::StaticIdentifier(identifier) => Some(identifier.name.to_string()),
        PropertyKey::StringLiteral(literal) => Some(literal.value.to_string()),
        _ => None,
    }
}

pub fn project_objects<'a>(root: &'a ObjectExpression<'a>) -> Vec<&'a ObjectExpression<'a>> {
    let Some(Expression::ArrayExpression(projects)) = property_expression(root, "projects") else {
        return Vec::new();
    };
    projects
        .elements
        .iter()
        .filter_map(array_element_object)
        .collect()
}

fn array_element_object<'a>(
    element: &'a oxc_ast::ast::ArrayExpressionElement<'a>,
) -> Option<&'a ObjectExpression<'a>> {
    match element {
        oxc_ast::ast::ArrayExpressionElement::ObjectExpression(object) => Some(object),
        _ => None,
    }
}
