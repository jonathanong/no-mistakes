use super::super::PostgresSqlExecution;
use sqlparser::ast::{Expr, UnaryOperator, UtilityOption, Value};

pub(super) fn execution(legacy: bool, options: Option<&[UtilityOption]>) -> PostgresSqlExecution {
    let mut analyze = legacy;
    let mut generic = false;
    let mut wal = false;
    let mut timing = false;
    let mut serialize = false;
    for option in options.into_iter().flatten() {
        let mut name = if option.name.quote_style.is_some() {
            option.name.value.clone()
        } else {
            option.name.value.to_ascii_lowercase()
        };
        if name == "analyse" && option.name.quote_style.is_none() {
            name = "analyze".into();
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
                "none" | "off" => serialize = false,
                "text" | "binary" => serialize = true,
                _ => return PostgresSqlExecution::Unknown,
            }
        } else if name == "wal" || name == "timing" {
            let Some(value) = boolean(option.arg.as_ref()) else {
                return PostgresSqlExecution::Unknown;
            };
            if name == "wal" {
                wal = value;
            } else {
                timing = value;
            }
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
    // PostgreSQL validates each occurrence, then applies cross-option checks to
    // the last value of each name; an earlier true flag can be reset to false.
    if analyze && generic || !analyze && (wal || timing || serialize) {
        PostgresSqlExecution::Unknown
    } else if analyze {
        PostgresSqlExecution::ExecutesForAnalysis
    } else {
        PostgresSqlExecution::NonExecuting
    }
}
fn boolean(expr: Option<&Expr>) -> Option<bool> {
    let expr = match expr {
        None => return Some(true),
        Some(expr) => expr,
    };
    match expr {
        Expr::Value(value) => match &value.value {
            Value::Boolean(value) => return Some(*value),
            Value::Number(value, _) => return integer(value, false),
            _ => {}
        },
        Expr::UnaryOp { op, expr } => {
            let Expr::Value(value) = expr.as_ref() else {
                return None;
            };
            let Value::Number(value, _) = &value.value else {
                return None;
            };
            return match op {
                UnaryOperator::Plus => integer(value, false),
                UnaryOperator::Minus => integer(value, true),
                _ => None,
            };
        }
        _ => {}
    }
    let value = string(expr)?;
    match value.to_ascii_lowercase().as_str() {
        "true" | "on" => Some(true),
        "false" | "off" => Some(false),
        _ => None,
    }
}

fn integer(value: &str, negative: bool) -> Option<bool> {
    match crate::codebase::postgres::numeric_literal::integer(value).ok()? {
        0 => Some(false),
        1 if !negative => Some(true),
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
            // All literal spellings have already been decoded by the request's
            // prepared token inventory. Decoding again would corrupt escapes.
            Value::SingleQuotedString(value)
            | Value::EscapedStringLiteral(value)
            | Value::UnicodeStringLiteral(value) => Some(value.clone()),
            Value::DollarQuotedString(value) => Some(value.value.clone()),
            _ => None,
        },
        _ => None,
    }
}

pub(in crate::codebase::postgres::source) fn prepare(
    tokens: &mut [sqlparser::tokenizer::TokenWithSpan],
) {
    use sqlparser::{keywords::Keyword, tokenizer::Token};
    let mut explain = false;
    let mut boundary = true;
    for token in tokens {
        if matches!(token.token, Token::Whitespace(_)) {
            continue;
        }
        if explain
            && matches!(&token.token, Token::Word(word) if word.quote_style.is_none() && word.value.eq_ignore_ascii_case("ANALYSE"))
        {
            token.token = Token::make_word("ANALYZE", None);
        }
        explain = boundary
            && matches!(&token.token, Token::Word(word) if word.quote_style.is_none() && word.keyword == Keyword::EXPLAIN);
        boundary = token.token == Token::SemiColon
            || matches!(&token.token, Token::Word(word) if word.quote_style.is_none() && [Keyword::BEGIN, Keyword::ATOMIC, Keyword::THEN, Keyword::ELSE].contains(&word.keyword));
    }
}
