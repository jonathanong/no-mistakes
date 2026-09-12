use super::{lexer, syntax};

pub(crate) struct ConditionFunctionCall<'a> {
    pub(crate) function: lexer::Function,
    pub(crate) arguments: Vec<&'a str>,
}

/// Parses a complete function-call expression after the shared expression
/// grammar has accepted it. Keeping argument boundaries here means condition
/// evaluation cannot mistake commas or parentheses inside valid nested calls
/// and escaped string literals for argument separators.
pub(crate) fn condition_function_call(expression: &str) -> Option<ConditionFunctionCall<'_>> {
    let expression = expression.trim();
    let tokens = lexer::tokenize(expression)?;
    syntax::parse(&tokens)?;
    let function = match tokens.as_slice() {
        [lexer::Token::Function(function), lexer::Token::LeftParen, ..] => *function,
        _ => return None,
    };
    let opening = expression.find('(')?;
    expression[..opening]
        .trim()
        .bytes()
        .all(|byte| {
            byte.is_ascii_alphanumeric()
                || byte.is_ascii_whitespace()
                || matches!(byte, b'_' | b'-')
        })
        .then_some(())?;
    let arguments = arguments(&expression[opening + 1..])?;
    Some(ConditionFunctionCall {
        function,
        arguments,
    })
}

fn arguments(body: &str) -> Option<Vec<&str>> {
    let bytes = body.as_bytes();
    let mut arguments = Vec::new();
    let mut start = 0;
    let mut index = 0;
    let mut depth = 0usize;
    let mut in_string = false;
    while index < bytes.len() {
        match bytes[index] {
            b'\'' if in_string && bytes.get(index + 1) == Some(&b'\'') => index += 2,
            b'\'' => {
                in_string = !in_string;
                index += 1;
            }
            b'(' | b'[' if !in_string => {
                depth += 1;
                index += 1;
            }
            b')' if !in_string && depth == 0 => {
                (body[index + 1..].trim().is_empty()).then_some(())?;
                let argument = body[start..index].trim();
                if !argument.is_empty() {
                    arguments.push(argument);
                }
                return Some(arguments);
            }
            b')' | b']' if !in_string => {
                depth = depth.checked_sub(1)?;
                index += 1;
            }
            b',' if !in_string && depth == 0 => {
                let argument = body[start..index].trim();
                (!argument.is_empty()).then_some(())?;
                arguments.push(argument);
                start = index + 1;
                index += 1;
            }
            _ => index += 1,
        }
    }
    None
}
