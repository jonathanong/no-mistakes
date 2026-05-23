use oxc_ast::ast::{
    Argument, BindingPattern, ExportDefaultDeclarationKind, Expression, ObjectExpression,
    ObjectPropertyKind, Program, Statement,
};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn stories_expression<'a>(program: &'a Program<'a>) -> Option<&'a Expression<'a>> {
    let bindings = top_level_object_bindings(program);
    default_export_object(program, &bindings).and_then(|object| object_property(object, "stories"))
}

fn top_level_object_bindings<'a>(program: &'a Program<'a>) -> BTreeMap<String, &'a Expression<'a>> {
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

fn binding_identifier_name<'a>(binding: &'a BindingPattern<'a>) -> Option<&'a str> {
    match binding {
        BindingPattern::BindingIdentifier(identifier) => Some(identifier.name.as_str()),
        _ => None,
    }
}

fn default_export_object<'a>(
    program: &'a Program<'a>,
    bindings: &BTreeMap<String, &'a Expression<'a>>,
) -> Option<&'a ObjectExpression<'a>> {
    for statement in &program.body {
        if let Statement::ExportDefaultDeclaration(export) = statement {
            return export_config_object(&export.declaration, bindings);
        }
    }
    None
}

fn export_config_object<'a>(
    export: &'a ExportDefaultDeclarationKind<'a>,
    bindings: &BTreeMap<String, &'a Expression<'a>>,
) -> Option<&'a ObjectExpression<'a>> {
    match export {
        ExportDefaultDeclarationKind::ObjectExpression(object) => Some(object),
        ExportDefaultDeclarationKind::CallExpression(call) => {
            call.arguments.first().and_then(|arg| {
                let mut seen = BTreeSet::new();
                argument_config_object(arg, bindings, &mut seen)
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
        _ => None,
    }
}

fn expression_config_object<'a>(
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
            .and_then(|arg| argument_config_object(arg, bindings, seen)),
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

fn object_property<'a>(object: &'a ObjectExpression<'a>, name: &str) -> Option<&'a Expression<'a>> {
    object.properties.iter().find_map(|property| {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            return None;
        };
        if property.computed || property.method {
            return None;
        }
        let key = crate::codebase::ts_source::static_property_key_name(&property.key)?;
        (key == name).then_some(&property.value)
    })
}
