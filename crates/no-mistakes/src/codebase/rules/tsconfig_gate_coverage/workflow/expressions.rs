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
    let mut remaining = value;
    loop {
        let Some(start) = remaining.find("${{") else {
            return !remaining.contains("}}");
        };
        let body = &remaining[start + "${{".len()..];
        let Some(end) = interpolated_expression_end(body) else {
            return false;
        };
        if lexer::tokenize(body[..end].trim())
            .and_then(|tokens| syntax::parse(&tokens))
            .is_none()
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
