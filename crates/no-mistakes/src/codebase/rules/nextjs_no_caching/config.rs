use super::patterns::boolean_value;
use crate::codebase::ts_source::static_property_key_name;
use oxc_ast::ast::{Argument, CallExpression, ObjectExpression, ObjectPropertyKind};

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

fn next_config_message(name: &str) -> String {
    format!("Next.js `{name}` config is disabled; remove static caching")
}
