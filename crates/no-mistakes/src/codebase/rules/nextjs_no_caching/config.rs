use super::patterns::boolean_value;
use crate::codebase::ts_source::static_property_key_name;
use oxc_ast::ast::{
    Argument, AssignmentExpression, AssignmentTarget, CallExpression, ExportDefaultDeclarationKind,
    Expression, ObjectExpression, ObjectPropertyKind,
};
use std::collections::HashMap;

pub(super) fn object_findings(obj: &ObjectExpression<'_>) -> Vec<(u32, String)> {
    let mut findings = Vec::new();
    for prop in &obj.properties {
        let ObjectPropertyKind::ObjectProperty(prop) = prop else {
            continue;
        };
        let Some(name) = static_property_key_name(&prop.key) else {
            continue;
        };
        match name {
            "cacheComponents" if boolean_value(&prop.value) == Some(true) => findings.push((
                prop.span.start,
                "Next.js cacheComponents config is disabled; remove static caching".to_string(),
            )),
            "cacheLife" | "cacheHandlers" => {
                findings.push((prop.span.start, next_config_message(name)));
            }
            "experimental" => {
                if let Expression::ObjectExpression(obj) = &prop.value {
                    findings.extend(object_findings(obj));
                }
            }
            _ => {}
        }
    }
    findings
}

pub(super) fn call_findings(call: &CallExpression<'_>) -> Vec<(u32, String)> {
    call.arguments
        .iter()
        .filter_map(|argument| match argument {
            Argument::ObjectExpression(obj) => Some(object_findings(obj)),
            _ => None,
        })
        .flatten()
        .collect()
}

pub(super) fn call_findings_with_bindings(
    call: &CallExpression<'_>,
    bindings: &HashMap<String, Vec<(u32, String)>>,
) -> Vec<(u32, String)> {
    call.arguments
        .iter()
        .flat_map(|argument| argument_findings(argument, bindings))
        .collect()
}

pub(super) fn expression_findings(expr: &Expression<'_>) -> Vec<(u32, String)> {
    match expr {
        Expression::ObjectExpression(obj) => object_findings(obj),
        Expression::CallExpression(call) => call_findings(call),
        Expression::ParenthesizedExpression(expr) => expression_findings(&expr.expression),
        Expression::TSAsExpression(expr) => expression_findings(&expr.expression),
        Expression::TSSatisfiesExpression(expr) => expression_findings(&expr.expression),
        _ => Vec::new(),
    }
}

pub(super) fn assignment_findings(
    assignment: &AssignmentExpression<'_>,
    bindings: &HashMap<String, Vec<(u32, String)>>,
) -> Vec<(u32, String)> {
    if assignment_target_path(&assignment.left)
        .as_deref()
        .is_none_or(|parts| parts != ["module", "exports"])
    {
        return Vec::new();
    }
    expression_findings_with_bindings(&assignment.right, bindings)
}

pub(super) fn default_export_findings(
    export: &ExportDefaultDeclarationKind<'_>,
    bindings: &HashMap<String, Vec<(u32, String)>>,
) -> Vec<(u32, String)> {
    match export {
        ExportDefaultDeclarationKind::Identifier(id) => {
            bindings.get(id.name.as_str()).cloned().unwrap_or_default()
        }
        ExportDefaultDeclarationKind::CallExpression(call) => {
            call_findings_with_bindings(call, bindings)
        }
        ExportDefaultDeclarationKind::ObjectExpression(obj) => object_findings(obj),
        ExportDefaultDeclarationKind::ParenthesizedExpression(expr) => {
            expression_findings(&expr.expression)
        }
        ExportDefaultDeclarationKind::TSAsExpression(expr) => expression_findings(&expr.expression),
        ExportDefaultDeclarationKind::TSSatisfiesExpression(expr) => {
            expression_findings(&expr.expression)
        }
        _ => Vec::new(),
    }
}

fn expression_findings_with_bindings(
    expr: &Expression<'_>,
    bindings: &HashMap<String, Vec<(u32, String)>>,
) -> Vec<(u32, String)> {
    match expr {
        Expression::Identifier(id) => bindings.get(id.name.as_str()).cloned().unwrap_or_default(),
        Expression::CallExpression(call) => call_findings_with_bindings(call, bindings),
        Expression::ParenthesizedExpression(expr) => {
            expression_findings_with_bindings(&expr.expression, bindings)
        }
        Expression::TSAsExpression(expr) => {
            expression_findings_with_bindings(&expr.expression, bindings)
        }
        Expression::TSSatisfiesExpression(expr) => {
            expression_findings_with_bindings(&expr.expression, bindings)
        }
        _ => expression_findings(expr),
    }
}

fn argument_findings(
    argument: &Argument<'_>,
    bindings: &HashMap<String, Vec<(u32, String)>>,
) -> Vec<(u32, String)> {
    match argument {
        Argument::Identifier(id) => bindings.get(id.name.as_str()).cloned().unwrap_or_default(),
        Argument::ObjectExpression(obj) => object_findings(obj),
        Argument::ParenthesizedExpression(expr) => {
            expression_findings_with_bindings(&expr.expression, bindings)
        }
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
