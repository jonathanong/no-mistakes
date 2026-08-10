mod calls;
mod contexts;
mod lexer;
mod syntax;

pub(crate) use calls::condition_function_call;
pub(crate) use lexer::Function;

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

pub(super) fn complete_expression_may_produce_mapping(value: &str) -> bool {
    let value = value.trim();
    let Some(body) = value
        .strip_prefix("${{")
        .and_then(|value| value.strip_suffix("}}"))
        .map(str::trim)
    else {
        return false;
    };
    lexer::tokenize(body)
        .and_then(|tokens| syntax::may_produce_mapping(&tokens))
        .unwrap_or(false)
}

/// Returns whether every root context referenced by a complete expression is
/// available at a workflow key. GitHub evaluates context availability before
/// the expression itself, so syntactically valid unavailable contexts must not
/// make a reusable-call binding appear executable.
pub(super) fn complete_expression_contexts_available(value: &str, allowed: &[&str]) -> bool {
    complete_expression_contexts_with_hash_files_available(value, allowed, false)
}

pub(super) fn complete_expression_contexts_with_hash_files_available(
    value: &str,
    allowed: &[&str],
    hash_files_available: bool,
) -> bool {
    let value = value.trim();
    let Some(body) = value
        .strip_prefix("${{")
        .and_then(|value| value.strip_suffix("}}"))
        .map(str::trim)
    else {
        return false;
    };
    let tokens = match lexer::tokenize(body) {
        Some(tokens) => tokens,
        None => return false,
    };
    syntax::parse(&tokens).is_some()
        && contexts::root_contexts_available(body, allowed)
        && special_functions_available(&tokens, hash_files_available, false)
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

pub(super) fn condition_has_status_function(value: &str) -> bool {
    let value = value.trim();
    let body = if value.starts_with("${{") || value.ends_with("}}") {
        value
            .strip_prefix("${{")
            .and_then(|body| body.strip_suffix("}}"))
            .map(str::trim)
    } else {
        Some(value)
    };
    body.and_then(lexer::tokenize).is_some_and(|tokens| {
        syntax::parse(&tokens).is_some()
            && tokens.iter().any(|token| {
                matches!(
                    token,
                    lexer::Token::Function(
                        lexer::Function::Success
                            | lexer::Function::Failure
                            | lexer::Function::Always
                            | lexer::Function::Cancelled
                    )
                )
            })
    })
}

pub(super) fn condition_expression_contexts_available(
    value: &str,
    allowed: &[&str],
    hash_files_available: bool,
) -> bool {
    let expression = value.trim();
    let expression = if expression.starts_with("${{") || expression.ends_with("}}") {
        expression
            .strip_prefix("${{")
            .and_then(|body| body.strip_suffix("}}"))
            .map(str::trim)
    } else {
        Some(expression)
    };
    expression.is_some_and(|expression| {
        let Some(tokens) = lexer::tokenize(expression) else {
            return false;
        };
        syntax::parse(&tokens).is_some()
            && contexts::root_contexts_available(expression, allowed)
            && special_functions_available(&tokens, hash_files_available, true)
    })
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
        let Some(tokens) = lexer::tokenize(expression) else {
            return false;
        };
        if syntax::parse(&tokens).is_none()
            || allowed
                .is_some_and(|allowed| !contexts::root_contexts_available(expression, allowed))
            || (allowed.is_some() && !special_functions_available(&tokens, false, false))
        {
            return false;
        }
        remaining = &body[end + "}}".len()..];
    }
}

fn special_functions_available(
    tokens: &[lexer::Token],
    hash_files_available: bool,
    status_functions_available: bool,
) -> bool {
    tokens.iter().all(|token| match token {
        lexer::Token::Function(lexer::Function::HashFiles) => hash_files_available,
        lexer::Token::Function(
            lexer::Function::Success
            | lexer::Function::Failure
            | lexer::Function::Always
            | lexer::Function::Cancelled,
        ) => status_functions_available,
        _ => true,
    })
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
