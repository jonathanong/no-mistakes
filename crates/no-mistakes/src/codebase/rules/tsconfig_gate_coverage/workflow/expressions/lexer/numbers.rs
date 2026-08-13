pub(super) fn starts(bytes: &[u8], index: usize) -> bool {
    bytes[index].is_ascii_digit()
        || (bytes[index] == b'-' && bytes.get(index + 1).is_some_and(u8::is_ascii_digit))
}

pub(super) fn end(bytes: &[u8], mut index: usize) -> Option<usize> {
    if bytes[index] == b'-' {
        index += 1;
    }
    if bytes.get(index) == Some(&b'0') && matches!(bytes.get(index + 1), Some(b'x' | b'X')) {
        index += 2;
        let start = index;
        while bytes.get(index).is_some_and(u8::is_ascii_hexdigit) {
            index += 1;
        }
        return (index > start).then_some(index);
    }
    let integer_start = index;
    while bytes.get(index).is_some_and(u8::is_ascii_digit) {
        index += 1;
    }
    if index == integer_start {
        return None;
    }
    if bytes.get(index) == Some(&b'.') {
        index += 1;
        let fractional_start = index;
        while bytes.get(index).is_some_and(u8::is_ascii_digit) {
            index += 1;
        }
        if index == fractional_start {
            return None;
        }
    }
    if matches!(bytes.get(index), Some(b'e' | b'E')) {
        index += 1;
        if matches!(bytes.get(index), Some(b'+' | b'-')) {
            index += 1;
        }
        let exponent_start = index;
        while bytes.get(index).is_some_and(u8::is_ascii_digit) {
            index += 1;
        }
        if index == exponent_start {
            return None;
        }
    }
    Some(index)
}
