use super::const_values::{property_name, static_value, ConstMap, StaticValue};
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

pub(super) fn fetch_cache_findings(
    options: &ObjectExpression<'_>,
    scopes: &[ConstMap],
) -> Vec<String> {
    let mut findings = Vec::new();
    for prop in &options.properties {
        let ObjectPropertyKind::ObjectProperty(prop) = prop else {
            continue;
        };
        let Some(name) = property_name(&prop.key) else {
            continue;
        };
        match name.as_str() {
            "cache" if resolved_string(&prop.value, scopes).as_deref() == Some("force-cache") => {
                findings.push(
                    "fetch cache: \"force-cache\" is disabled; use uncached request-time data"
                        .to_string(),
                );
            }
            "next" => {
                let Expression::ObjectExpression(next) = &prop.value else {
                    continue;
                };
                findings.extend(next_fetch_cache_findings(next, scopes));
            }
            _ => {}
        }
    }
    findings
}

fn next_fetch_cache_findings(next: &ObjectExpression<'_>, scopes: &[ConstMap]) -> Vec<String> {
    let mut findings = Vec::new();
    for prop in &next.properties {
        let ObjectPropertyKind::ObjectProperty(prop) = prop else {
            continue;
        };
        let Some(name) = property_name(&prop.key) else {
            continue;
        };
        match name.as_str() {
            "revalidate" if banned_revalidate_value(&prop.value, scopes) => {
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

pub(super) fn banned_segment_config(
    name: &str,
    value: &Expression<'_>,
    scopes: &[ConstMap],
) -> bool {
    match name {
        "revalidate" => banned_revalidate_value(value, scopes),
        "fetchCache" => matches!(
            resolved_string(value, scopes).as_deref(),
            Some("force-cache" | "only-cache" | "default-cache")
        ),
        "dynamic" => matches!(
            resolved_string(value, scopes).as_deref(),
            Some("force-static" | "error")
        ),
        _ => false,
    }
}

fn banned_revalidate_value(value: &Expression<'_>, scopes: &[ConstMap]) -> bool {
    match static_value(value, scopes) {
        Some(StaticValue::Bool(enabled)) => !enabled,
        Some(StaticValue::Number(number)) => number > 0.0,
        _ => false,
    }
}

fn resolved_string(value: &Expression<'_>, scopes: &[ConstMap]) -> Option<String> {
    match static_value(value, scopes)? {
        StaticValue::String(text) => Some(text),
        _ => None,
    }
}

pub(super) fn single_binding_name(pattern: &BindingPattern<'_>) -> Option<String> {
    match pattern {
        BindingPattern::BindingIdentifier(id) => Some(id.name.to_string()),
        _ => None,
    }
}
