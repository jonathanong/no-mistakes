use crate::codebase::ts_source::static_property_key_name;
use oxc_ast::ast::{BindingPattern, Expression, ObjectExpression, ObjectPropertyKind};

pub(super) fn banned_next_cache_import(name: &str) -> bool {
    matches!(
        name,
        "cacheLife"
            | "cacheTag"
            | "revalidatePath"
            | "revalidateTag"
            | "updateTag"
            | "unstable_cache"
    )
}

pub(super) fn fetch_cache_findings(options: &ObjectExpression<'_>) -> Vec<String> {
    let mut findings = Vec::new();
    for prop in &options.properties {
        let ObjectPropertyKind::ObjectProperty(prop) = prop else {
            continue;
        };
        let Some(name) = static_property_key_name(&prop.key) else {
            continue;
        };
        match name {
            "cache" if string_value(&prop.value) == Some("force-cache") => {
                findings.push(
                    "fetch cache: \"force-cache\" is disabled; use uncached request-time data"
                        .to_string(),
                );
            }
            "next" => {
                let Expression::ObjectExpression(next) = &prop.value else {
                    continue;
                };
                findings.extend(next_fetch_cache_findings(next));
            }
            _ => {}
        }
    }
    findings
}

fn next_fetch_cache_findings(next: &ObjectExpression<'_>) -> Vec<String> {
    let mut findings = Vec::new();
    for prop in &next.properties {
        let ObjectPropertyKind::ObjectProperty(prop) = prop else {
            continue;
        };
        let Some(name) = static_property_key_name(&prop.key) else {
            continue;
        };
        match name {
            "revalidate" if banned_revalidate_value(&prop.value) => {
                findings.push(
                    "fetch next.revalidate caching is disabled; use revalidate: 0 or no-store"
                        .to_string(),
                );
            }
            "tags" if matches!(prop.value, Expression::ArrayExpression(_)) => {
                findings.push("fetch next.tags caching is disabled; remove cache tags".to_string());
            }
            _ => {}
        }
    }
    findings
}

pub(super) fn banned_segment_config(name: &str, value: &Expression<'_>) -> bool {
    match name {
        "revalidate" => banned_revalidate_value(value),
        "fetchCache" => matches!(
            string_value(value),
            Some("force-cache" | "only-cache" | "default-cache")
        ),
        "dynamic" => matches!(string_value(value), Some("force-static" | "error")),
        _ => false,
    }
}

fn banned_revalidate_value(value: &Expression<'_>) -> bool {
    match value {
        Expression::BooleanLiteral(boolean) => !boolean.value,
        Expression::NumericLiteral(number) => number.value > 0.0,
        _ => false,
    }
}

fn string_value<'a>(value: &'a Expression<'a>) -> Option<&'a str> {
    match value {
        Expression::StringLiteral(literal) => Some(literal.value.as_str()),
        _ => None,
    }
}

pub(super) fn boolean_value(value: &Expression<'_>) -> Option<bool> {
    match value {
        Expression::BooleanLiteral(literal) => Some(literal.value),
        _ => None,
    }
}

pub(super) fn single_binding_name(pattern: &BindingPattern<'_>) -> Option<String> {
    match pattern {
        BindingPattern::BindingIdentifier(id) => Some(id.name.to_string()),
        BindingPattern::AssignmentPattern(assign) => single_binding_name(&assign.left),
        _ => None,
    }
}
