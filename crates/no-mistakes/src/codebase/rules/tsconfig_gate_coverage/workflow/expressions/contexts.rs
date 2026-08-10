use super::lexer;

pub(super) fn root_contexts_available(expression: &str, allowed: &[&str]) -> bool {
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
