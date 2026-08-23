use super::Comparison;

pub(super) fn top_level_operator(expression: &str, operator: &[u8; 2]) -> Option<usize> {
    let bytes = expression.as_bytes();
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
            b')' | b']' if !in_string => {
                depth = depth.saturating_sub(1);
                index += 1;
            }
            _ if !in_string && depth == 0 && bytes.get(index..index + 2) == Some(operator) => {
                return Some(index);
            }
            _ => index += 1,
        }
    }
    None
}

pub(super) fn top_level_comparison(expression: &str) -> Option<(usize, usize, Comparison)> {
    let bytes = expression.as_bytes();
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
            b')' | b']' if !in_string => {
                depth = depth.saturating_sub(1);
                index += 1;
            }
            b'=' if !in_string && depth == 0 && bytes.get(index + 1) == Some(&b'=') => {
                return Some((index, 2, Comparison::Equal));
            }
            b'!' if !in_string && depth == 0 && bytes.get(index + 1) == Some(&b'=') => {
                return Some((index, 2, Comparison::NotEqual));
            }
            b'<' if !in_string && depth == 0 => {
                let equal = bytes.get(index + 1) == Some(&b'=');
                return Some((
                    index,
                    1 + usize::from(equal),
                    if equal {
                        Comparison::LessThanOrEqual
                    } else {
                        Comparison::LessThan
                    },
                ));
            }
            b'>' if !in_string && depth == 0 => {
                let equal = bytes.get(index + 1) == Some(&b'=');
                return Some((
                    index,
                    1 + usize::from(equal),
                    if equal {
                        Comparison::GreaterThanOrEqual
                    } else {
                        Comparison::GreaterThan
                    },
                ));
            }
            _ => index += 1,
        }
    }
    None
}
