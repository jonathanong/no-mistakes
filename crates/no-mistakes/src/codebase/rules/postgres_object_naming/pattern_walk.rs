pub(super) fn ends_unescaped_dollar(pattern: &str) -> bool {
    let bytes = pattern.as_bytes();
    if bytes.last() != Some(&b'$') {
        return false;
    }
    let mut slashes = 0;
    let mut index = bytes.len() - 1;
    while index > 0 && bytes[index - 1] == b'\\' {
        slashes += 1;
        index -= 1;
    }
    slashes % 2 == 0
}

pub(super) fn placeholder_is_plain(pattern: &str) -> bool {
    let chars: Vec<(usize, char)> = pattern.char_indices().collect();
    let mut index = 0;
    let mut depth = 0i32;
    let mut in_class = false;
    let mut top_alt = false;
    let mut ok = false;
    while index < chars.len() {
        let (byte, character) = chars[index];
        if character == '\\' {
            index += 2;
            continue;
        }
        if in_class {
            if pattern[byte..].starts_with("{table}") {
                return false;
            }
            if character == ']' {
                in_class = false;
            }
            index += 1;
            continue;
        }
        if character == '[' {
            index = open_class(&chars, index);
            in_class = true;
            continue;
        }
        if character == '(' {
            depth += 1;
        }
        if character == ')' && depth > 0 {
            depth -= 1;
        }
        if character == '|' && depth == 0 {
            top_alt = true;
        }
        if pattern[byte..].starts_with("{table}") {
            if depth != 0 || in_class {
                return false;
            }
            ok = true;
        }
        index += 1;
    }
    ok && !top_alt
}

fn open_class(chars: &[(usize, char)], mut index: usize) -> usize {
    index += 1;
    if chars.get(index).is_some_and(|(_, char)| *char == '^') {
        index += 1;
    }
    if chars.get(index).is_some_and(|(_, char)| *char == ']') {
        index += 1;
    }
    index
}
