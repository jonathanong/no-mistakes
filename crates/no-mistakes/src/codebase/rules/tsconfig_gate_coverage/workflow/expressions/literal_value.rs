use super::{calls::condition_function_call, lexer, syntax};
use serde_yaml::Value;

/// Returns the typed YAML value for a context-free expression whose result is
/// statically available: a literal (possibly parenthesized) or `fromJSON`
/// applied to one string literal. Other expressions remain unknown.
pub(in super::super) fn complete_literal_expression_value(value: &str) -> Option<Value> {
    let value = value.trim();
    let body = value.strip_prefix("${{")?.strip_suffix("}}")?.trim();
    let tokens = lexer::tokenize(body)?;
    syntax::parse(&tokens)?;
    if tokens.iter().all(|token| {
        matches!(
            token,
            lexer::Token::Boolean
                | lexer::Token::Number
                | lexer::Token::String
                | lexer::Token::Null
                | lexer::Token::LeftParen
                | lexer::Token::RightParen
        )
    }) {
        return serde_yaml::from_str(strip_literal_parentheses(body)).ok();
    }
    literal_from_json_value(body)
}

/// Returns a parsed JSON value when a complete, context-free `fromJSON` call
/// has one literal string argument. This accepts an expression body so condition
/// evaluation can use it after removing `${{ ... }}` delimiters.
pub(in super::super) fn literal_from_json_value(expression: &str) -> Option<Value> {
    let call = condition_function_call(expression)?;
    (call.function == lexer::Function::FromJson && call.arguments.len() == 1).then_some(())?;
    let encoded = github_string_literal(call.arguments[0])?;
    serde_json::from_str::<serde_json::Value>(&encoded)
        .ok()
        .and_then(|value| serde_yaml::to_value(value).ok())
}

pub(in super::super) fn invalid_literal_from_json(value: &str) -> bool {
    let value = value.trim();
    let Some(body) = value
        .strip_prefix("${{")
        .and_then(|value| value.strip_suffix("}}"))
        .map(str::trim)
    else {
        return false;
    };
    invalid_literal_from_json_in(body)
}

fn invalid_literal_from_json_in(expression: &str) -> bool {
    let bytes = expression.as_bytes();
    let mut index = 0;
    let mut in_string = false;
    while index < bytes.len() {
        match bytes[index] {
            b'\'' if in_string && bytes.get(index + 1) == Some(&b'\'') => index += 2,
            b'\'' => {
                in_string = !in_string;
                index += 1;
            }
            byte if !in_string && (byte.is_ascii_alphabetic() || byte == b'_') => {
                let start = index;
                index += 1;
                while bytes
                    .get(index)
                    .is_some_and(|byte| byte.is_ascii_alphanumeric() || *byte == b'_')
                {
                    index += 1;
                }
                if expression[start..index].eq_ignore_ascii_case("fromjson") {
                    let mut opening = index;
                    while bytes.get(opening).is_some_and(u8::is_ascii_whitespace) {
                        opening += 1;
                    }
                    if bytes.get(opening) == Some(&b'(')
                        && matching_parenthesis(expression, opening).is_some_and(|closing| {
                            condition_function_call(&expression[start..=closing]).is_some_and(
                                |call| {
                                    github_string_literal(call.arguments[0]).is_some_and(
                                        |encoded| {
                                            serde_json::from_str::<serde_json::Value>(&encoded)
                                                .is_err()
                                        },
                                    )
                                },
                            )
                        })
                    {
                        return true;
                    }
                }
            }
            _ => index += 1,
        }
    }
    false
}

fn matching_parenthesis(expression: &str, opening: usize) -> Option<usize> {
    let bytes = expression.as_bytes();
    let mut depth = 0usize;
    let mut index = opening;
    let mut in_string = false;
    while index < bytes.len() {
        match bytes[index] {
            b'\'' if in_string && bytes.get(index + 1) == Some(&b'\'') => index += 2,
            b'\'' => {
                in_string = !in_string;
                index += 1;
            }
            b'(' if !in_string => {
                depth += 1;
                index += 1;
            }
            b')' if !in_string => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(index);
                }
                index += 1;
            }
            _ => index += 1,
        }
    }
    None
}

fn github_string_literal(value: &str) -> Option<String> {
    let value = value.trim();
    let body = value.strip_prefix('\'')?.strip_suffix('\'')?;
    Some(body.replace("''", "'"))
}

fn strip_literal_parentheses(mut value: &str) -> &str {
    while value.starts_with('(') && value.ends_with(')') {
        value = value[1..value.len() - 1].trim();
    }
    value
}
