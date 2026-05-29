use crate::ast;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{
    Argument, AssignmentTarget, ExportDefaultDeclarationKind, Expression, ObjectExpression,
    Program, Statement,
};
use std::collections::{BTreeMap, BTreeSet};

pub(in crate::integration_tests) fn default_export_object<'a>(
    program: &'a Program<'a>,
    bindings: &BTreeMap<String, &'a Expression<'a>>,
) -> Option<&'a ObjectExpression<'a>> {
    for statement in &program.body {
        if let Statement::ExportDefaultDeclaration(export) = statement {
            return export_config_object(&export.declaration, bindings);
        }
        if let Some(object) = commonjs_config_object(statement, bindings) {
            return Some(object);
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
        ExportDefaultDeclarationKind::TSAsExpression(expression) => {
            let mut seen = BTreeSet::new();
            expression_config_object(&expression.expression, bindings, &mut seen)
        }
        ExportDefaultDeclarationKind::TSSatisfiesExpression(expression) => {
            let mut seen = BTreeSet::new();
            expression_config_object(&expression.expression, bindings, &mut seen)
        }
        ExportDefaultDeclarationKind::TSTypeAssertion(expression) => {
            let mut seen = BTreeSet::new();
            expression_config_object(&expression.expression, bindings, &mut seen)
        }
        ExportDefaultDeclarationKind::TSNonNullExpression(expression) => {
            let mut seen = BTreeSet::new();
            expression_config_object(&expression.expression, bindings, &mut seen)
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
        _ => None,
    }
}

fn expression_config_object<'a>(
    expression: &'a Expression<'a>,
    bindings: &BTreeMap<String, &'a Expression<'a>>,
    seen: &mut BTreeSet<String>,
) -> Option<&'a ObjectExpression<'a>> {
    match unwrap_ts_wrappers(expression) {
        Expression::CallExpression(call) => call
            .arguments
            .first()
            .and_then(|argument| argument_config_object(argument, bindings, seen)),
        Expression::Identifier(identifier) => {
            identifier_config_object(identifier.name.as_str(), bindings, seen)
        }
        Expression::ObjectExpression(object) => Some(object),
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
