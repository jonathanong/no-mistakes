use super::{lexer, syntax};
use serde_yaml::Value;

/// Returns the typed YAML value for a complete expression that is only a
/// literal (possibly wrapped in parentheses). Other complete expressions may
/// depend on runtime state and must remain unknown to static callers.
pub(in super::super) fn complete_literal_expression_value(value: &str) -> Option<Value> {
    let value = value.trim();
    let body = value.strip_prefix("${{")?.strip_suffix("}}")?.trim();
    let tokens = lexer::tokenize(body)?;
    syntax::parse(&tokens)?;
    if !tokens.iter().all(|token| {
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
        return None;
    }
    serde_yaml::from_str(strip_literal_parentheses(body)).ok()
}

fn strip_literal_parentheses(mut value: &str) -> &str {
    while value.starts_with('(') && value.ends_with(')') {
        value = value[1..value.len() - 1].trim();
    }
    value
}
