use super::super::PostgresSqlExecution;
use sqlparser::ast::{Expr, UtilityOption, Value};

pub(super) fn execution(legacy: bool, options: Option<&[UtilityOption]>) -> PostgresSqlExecution {
    let mut analyze = legacy;
    let mut seen = std::collections::BTreeSet::new();
    let mut generic = false;
    for option in options.into_iter().flatten() {
        if option.name.quote_style.is_some() {
            return PostgresSqlExecution::Unknown;
        }
        let name = option.name.value.to_ascii_lowercase();
        if !seen.insert(name.clone()) {
            return PostgresSqlExecution::Unknown;
        }
        if name == "analyze" {
            let Some(value) = boolean(option.arg.as_ref()) else {
                return PostgresSqlExecution::Unknown;
            };
            analyze = value;
        } else if name == "generic_plan" {
            let Some(value) = boolean(option.arg.as_ref()) else {
                return PostgresSqlExecution::Unknown;
            };
            generic = value;
        } else if [
            "verbose", "costs", "settings", "buffers", "wal", "timing", "summary", "memory",
        ]
        .contains(&name.as_str())
        {
            if boolean(option.arg.as_ref()).is_none() {
                return PostgresSqlExecution::Unknown;
            }
        } else if name == "format" {
            let value = option
                .arg
                .as_ref()
                .map(ToString::to_string)
                .unwrap_or_default()
                .to_ascii_lowercase();
            if ![
                "text", "xml", "json", "yaml", "'text'", "'xml'", "'json'", "'yaml'",
            ]
            .contains(&value.as_str())
            {
                return PostgresSqlExecution::Unknown;
            }
        } else {
            return PostgresSqlExecution::Unknown;
        }
    }
    if analyze && generic {
        PostgresSqlExecution::Unknown
    } else if analyze {
        PostgresSqlExecution::ExecutesForAnalysis
    } else {
        PostgresSqlExecution::NonExecuting
    }
}
fn boolean(expr: Option<&Expr>) -> Option<bool> {
    let value = match expr {
        None => return Some(true),
        Some(Expr::Value(value)) => match &value.value {
            Value::Boolean(value) => return Some(*value),
            Value::Number(value, _) | Value::SingleQuotedString(value) => value.as_str(),
            _ => return None,
        },
        Some(Expr::Identifier(value)) if value.quote_style.is_none() => value.value.as_str(),
        _ => return None,
    };
    match value.to_ascii_lowercase().as_str() {
        "true" | "on" | "1" => Some(true),
        "false" | "off" | "0" => Some(false),
        _ => None,
    }
}
