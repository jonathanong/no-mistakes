use crate::ast;
use oxc_ast::ast::{
    AssignmentTarget, ExportDefaultDeclarationKind, Expression, ObjectExpression, Program,
    Statement,
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
                super::argument_config_object(arg, bindings, &mut seen)
            })
        }
        ExportDefaultDeclarationKind::Identifier(identifier) => {
            let mut seen = BTreeSet::new();
            super::identifier_config_object(identifier.name.as_str(), bindings, &mut seen)
        }
        ExportDefaultDeclarationKind::ParenthesizedExpression(parenthesized) => {
            let mut seen = BTreeSet::new();
            super::expression_config_object(&parenthesized.expression, bindings, &mut seen)
        }
        ExportDefaultDeclarationKind::TSAsExpression(expression) => {
            let mut seen = BTreeSet::new();
            super::expression_config_object(&expression.expression, bindings, &mut seen)
        }
        ExportDefaultDeclarationKind::TSSatisfiesExpression(expression) => {
            let mut seen = BTreeSet::new();
            super::expression_config_object(&expression.expression, bindings, &mut seen)
        }
        ExportDefaultDeclarationKind::TSTypeAssertion(expression) => {
            let mut seen = BTreeSet::new();
            super::expression_config_object(&expression.expression, bindings, &mut seen)
        }
        ExportDefaultDeclarationKind::TSNonNullExpression(expression) => {
            let mut seen = BTreeSet::new();
            super::expression_config_object(&expression.expression, bindings, &mut seen)
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
    super::expression_config_object(&assignment.right, bindings, &mut seen)
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
