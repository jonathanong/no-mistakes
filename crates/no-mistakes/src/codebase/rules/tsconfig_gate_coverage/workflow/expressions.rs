mod lexer;
mod syntax;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum StaticExpressionType {
    Boolean,
    Number,
    String,
    Null,
    Dynamic,
}

pub(super) fn complete_expression_type(value: &str) -> Option<StaticExpressionType> {
    let value = value.trim();
    let body = value.strip_prefix("${{")?.strip_suffix("}}")?.trim();
    syntax::parse(&lexer::tokenize(body)?)
}

/// Returns whether every root context referenced by a complete expression is
/// available at a workflow key. GitHub evaluates context availability before
/// the expression itself, so syntactically valid unavailable contexts must not
/// make a reusable-call binding appear executable.
pub(super) fn complete_expression_contexts_available(value: &str, allowed: &[&str]) -> bool {
    let value = value.trim();
    let Some(body) = value
        .strip_prefix("${{")
        .and_then(|value| value.strip_suffix("}}"))
        .map(str::trim)
    else {
        return false;
    };
    syntax::parse(&match lexer::tokenize(body) {
        Some(tokens) => tokens,
        None => return false,
    })
    .is_some()
        && root_contexts_available(body, allowed)
}

fn root_contexts_available(expression: &str, allowed: &[&str]) -> bool {
    let bytes = expression.as_bytes();
    let mut index = 0;
    let mut previous_non_whitespace = None;
    while index < bytes.len() {
        if bytes[index].is_ascii_whitespace() {
            index += 1;
            continue;
        }
        if bytes[index] == b'\'' {
            let Some(end) = quoted_string_end(bytes, index + 1) else {
                return false;
            };
            index = end;
            previous_non_whitespace = Some(b'\'');
            continue;
        }
        if let Some(end) = lexer::numeric_literal_end(bytes, index) {
            previous_non_whitespace = bytes.get(end.saturating_sub(1)).copied();
            index = end;
            continue;
        }
        if identifier_start(bytes[index]) {
            let start = index;
            index += 1;
            while index < bytes.len() && identifier_continue(bytes[index]) {
                index += 1;
            }
            let identifier = &expression[start..index];
            let next = bytes[index..]
                .iter()
                .copied()
                .find(|byte| !byte.is_ascii_whitespace());
            let is_property = previous_non_whitespace == Some(b'.');
            let is_function = next == Some(b'(');
            let is_literal = matches!(
                identifier.to_ascii_lowercase().as_str(),
                "true" | "false" | "null"
            );
            if !is_property
                && !is_function
                && !is_literal
                && !allowed
                    .iter()
                    .any(|context| identifier.eq_ignore_ascii_case(context))
            {
                return false;
            }
            previous_non_whitespace = Some(bytes[index - 1]);
            continue;
        }
        previous_non_whitespace = Some(bytes[index]);
        index += 1;
    }
    true
}

fn quoted_string_end(bytes: &[u8], mut index: usize) -> Option<usize> {
    while index < bytes.len() {
        if bytes[index] == b'\'' {
            if bytes.get(index + 1) == Some(&b'\'') {
                index += 2;
            } else {
                return Some(index + 1);
            }
        } else {
            index += 1;
        }
    }
    None
}

fn identifier_start(byte: u8) -> bool {
    byte.is_ascii_alphabetic() || byte == b'_'
}

fn identifier_continue(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-')
}

pub(super) fn condition_expression_valid(value: &str) -> bool {
    let value = value.trim();
    if value.starts_with("${{") || value.ends_with("}}") {
        complete_expression_type(value).is_some()
    } else {
        lexer::tokenize(value)
            .and_then(|tokens| syntax::parse(&tokens))
            .is_some()
    }
}

pub(super) fn interpolated_expression_valid(value: &str) -> bool {
    interpolated_expression_valid_for_contexts(value, None)
}

pub(super) fn interpolated_expression_contexts_available(value: &str, allowed: &[&str]) -> bool {
    interpolated_expression_valid_for_contexts(value, Some(allowed))
}

fn interpolated_expression_valid_for_contexts(value: &str, allowed: Option<&[&str]>) -> bool {
    let mut remaining = value;
    loop {
        let Some(start) = remaining.find("${{") else {
            return !remaining.contains("}}");
        };
        let body = &remaining[start + "${{".len()..];
        let Some(end) = interpolated_expression_end(body) else {
            return false;
        };
        let expression = body[..end].trim();
        if lexer::tokenize(expression)
            .and_then(|tokens| syntax::parse(&tokens))
            .is_none()
            || allowed.is_some_and(|allowed| !root_contexts_available(expression, allowed))
        {
            return false;
        }
        remaining = &body[end + "}}".len()..];
    }
}

fn interpolated_expression_end(value: &str) -> Option<usize> {
    let bytes = value.as_bytes();
    let mut index = 0;
    let mut in_string = false;
    while index < bytes.len() {
        match bytes[index] {
            b'\'' => {
                if in_string && bytes.get(index + 1) == Some(&b'\'') {
                    index += 2;
                } else {
                    in_string = !in_string;
                    index += 1;
                }
            }
            b'}' if !in_string && bytes.get(index + 1) == Some(&b'}') => return Some(index),
            _ => index += 1,
        }
    }
    None
}

#[cfg(test)]
mod tests;
