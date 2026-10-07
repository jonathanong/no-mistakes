use super::super::PostgresSqlExecution;
use sqlparser::ast::{Expr, UtilityOption, Value};

pub(super) fn execution(legacy: bool, options: Option<&[UtilityOption]>) -> PostgresSqlExecution {
    let mut analyze = legacy;
    let mut seen = std::collections::BTreeSet::new();
    let mut generic = false;
    let mut requires_analyze = false;
    for option in options.into_iter().flatten() {
        if option.name.quote_style.is_some() {
            return PostgresSqlExecution::Unknown;
        }
        let mut name = option.name.value.to_ascii_lowercase();
        if name == "analyse" {
            name = "analyze".into();
        }
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
        } else if name == "serialize" {
            let value = match option.arg.as_ref() {
                None => "text".into(),
                Some(expr) => match string(expr) {
                    Some(value) => value,
                    None => return PostgresSqlExecution::Unknown,
                },
            };
            match value.as_str() {
                "none" | "off" => {}
                "text" | "binary" => requires_analyze = true,
                _ => return PostgresSqlExecution::Unknown,
            }
        } else if name == "wal" || name == "timing" {
            let Some(value) = boolean(option.arg.as_ref()) else {
                return PostgresSqlExecution::Unknown;
            };
            requires_analyze |= value;
        } else if [
            "verbose", "costs", "settings", "buffers", "summary", "memory",
        ]
        .contains(&name.as_str())
        {
            if boolean(option.arg.as_ref()).is_none() {
                return PostgresSqlExecution::Unknown;
            }
        } else if name == "format" {
            let value = option.arg.as_ref().and_then(string);
            if !matches!(value.as_deref(), Some("text" | "xml" | "json" | "yaml")) {
                return PostgresSqlExecution::Unknown;
            }
        } else {
            return PostgresSqlExecution::Unknown;
        }
    }
    if analyze && generic || !analyze && requires_analyze {
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

pub(super) fn legacy_format(format: Option<sqlparser::ast::AnalyzeFormatKind>) -> bool {
    format.is_none()
}

// PostgreSQL folds identifiers, but compares literal option values case-sensitively.
fn string(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Identifier(value) if value.quote_style.is_none() => {
            Some(value.value.to_ascii_lowercase())
        }
        Expr::Identifier(value) => Some(value.value.clone()),
        Expr::Value(value) => match &value.value {
            Value::SingleQuotedString(value) => Some(value.clone()),
            _ => None,
        },
        _ => None,
    }
}
