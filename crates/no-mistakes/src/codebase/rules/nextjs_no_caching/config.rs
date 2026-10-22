use super::const_values::{property_name, resolved_bool, ConstMap};
use oxc_ast::ast::{
    Argument, AssignmentExpression, AssignmentTarget, CallExpression, ExportDefaultDeclarationKind,
    Expression, ObjectExpression, ObjectPropertyKind,
};
use std::collections::HashMap;

pub(super) fn object_findings(
    obj: &ObjectExpression<'_>,
    scopes: &[ConstMap],
) -> Vec<(u32, String)> {
    let mut findings = Vec::new();
    for prop in &obj.properties {
        let ObjectPropertyKind::ObjectProperty(prop) = prop else {
            continue;
        };
        let Some(name) = property_name(&prop.key) else {
            continue;
        };
        match name.as_str() {
            "cacheComponents" if resolved_bool(&prop.value, scopes) == Some(true) => {
                findings.push((
                    prop.span.start,
                    "Next.js cacheComponents config is disabled; remove static caching".to_string(),
                ))
            }
            "cacheLife" | "cacheHandlers" => {
                findings.push((prop.span.start, next_config_message(&name)));
            }
            "experimental" => {
                if let Expression::ObjectExpression(obj) = &prop.value {
                    findings.extend(object_findings(obj, scopes));
                }
            }
            _ => {}
        }
    }
    findings
}

pub(super) fn call_findings_with_bindings(
    call: &CallExpression<'_>,
    bindings: &HashMap<String, Vec<(u32, String)>>,
    scopes: &[ConstMap],
) -> Vec<(u32, String)> {
    call.arguments
        .iter()
        .flat_map(|argument| argument_findings(argument, bindings, scopes))
        .collect()
}

pub(super) fn expression_findings(
    expr: &Expression<'_>,
    scopes: &[ConstMap],
) -> Vec<(u32, String)> {
    match expr {
        Expression::ObjectExpression(obj) => object_findings(obj, scopes),
        Expression::ParenthesizedExpression(expr) => expression_findings(&expr.expression, scopes),
        Expression::TSAsExpression(expr) => expression_findings(&expr.expression, scopes),
        Expression::TSSatisfiesExpression(expr) => expression_findings(&expr.expression, scopes),
        _ => Vec::new(),
    }
}

pub(super) fn assignment_findings(
    assignment: &AssignmentExpression<'_>,
    bindings: &HashMap<String, Vec<(u32, String)>>,
    scopes: &[ConstMap],
) -> Vec<(u32, String)> {
    if assignment_target_path(&assignment.left)
        .as_deref()
        .is_none_or(|parts| parts != ["module", "exports"])
    {
        return Vec::new();
    }
    expression_findings_with_bindings(&assignment.right, bindings, scopes)
}

pub(super) fn default_export_findings(
    export: &ExportDefaultDeclarationKind<'_>,
    bindings: &HashMap<String, Vec<(u32, String)>>,
    scopes: &[ConstMap],
) -> Vec<(u32, String)> {
    match export {
        ExportDefaultDeclarationKind::Identifier(id) => {
            bindings.get(id.name.as_str()).cloned().unwrap_or_default()
        }
        ExportDefaultDeclarationKind::CallExpression(call) => {
            call_findings_with_bindings(call, bindings, scopes)
        }
        ExportDefaultDeclarationKind::ObjectExpression(obj) => object_findings(obj, scopes),
        ExportDefaultDeclarationKind::ParenthesizedExpression(expr) => {
            expression_findings(&expr.expression, scopes)
        }
        ExportDefaultDeclarationKind::TSAsExpression(expr) => {
            expression_findings(&expr.expression, scopes)
        }
        ExportDefaultDeclarationKind::TSSatisfiesExpression(expr) => {
            expression_findings(&expr.expression, scopes)
        }
        _ => Vec::new(),
    }
}

fn expression_findings_with_bindings(
    expr: &Expression<'_>,
    bindings: &HashMap<String, Vec<(u32, String)>>,
    scopes: &[ConstMap],
) -> Vec<(u32, String)> {
    match expr {
        Expression::Identifier(id) => bindings.get(id.name.as_str()).cloned().unwrap_or_default(),
        Expression::CallExpression(call) => call_findings_with_bindings(call, bindings, scopes),
        Expression::ParenthesizedExpression(expr) => {
            expression_findings_with_bindings(&expr.expression, bindings, scopes)
        }
        Expression::TSAsExpression(expr) => {
            expression_findings_with_bindings(&expr.expression, bindings, scopes)
        }
        Expression::TSSatisfiesExpression(expr) => {
            expression_findings_with_bindings(&expr.expression, bindings, scopes)
        }
        _ => expression_findings(expr, scopes),
    }
}

fn argument_findings(
    argument: &Argument<'_>,
    bindings: &HashMap<String, Vec<(u32, String)>>,
    scopes: &[ConstMap],
) -> Vec<(u32, String)> {
    match argument {
        Argument::Identifier(id) => bindings.get(id.name.as_str()).cloned().unwrap_or_default(),
        Argument::ObjectExpression(obj) => object_findings(obj, scopes),
        _ => Vec::new(),
    }
}

fn assignment_target_path(target: &AssignmentTarget<'_>) -> Option<Vec<String>> {
    match target {
        AssignmentTarget::StaticMemberExpression(member) => {
            let mut parts = crate::ast::expression_path(&member.object)?;
            parts.push(member.property.name.to_string());
            Some(parts)
        }
        _ => None,
    }
}

fn next_config_message(name: &str) -> String {
    format!("Next.js `{name}` config is disabled; remove static caching")
}
