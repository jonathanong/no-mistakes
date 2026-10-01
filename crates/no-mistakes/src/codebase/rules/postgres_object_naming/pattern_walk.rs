pub(super) fn active_flags(pre: &str) -> String {
    let chars: Vec<char> = pre.chars().collect();
    let mut flags = String::new();
    let mut depth = 0i32;
    let mut index = 0;
    let mut in_class = false;
    while index < chars.len() {
        let character = chars[index];
        if character == '\\' {
            index += 2;
            continue;
        }
        if in_class {
            if character == ']' {
                in_class = false;
            }
            index += 1;
            continue;
        }
        if character == '[' {
            index += 1;
            if chars.get(index) == Some(&'^') {
                index += 1;
            }
            if chars.get(index) == Some(&']') {
                index += 1;
            }
            in_class = true;
            continue;
        }
        if character == '(' {
            if let Some(end) = persistent_flag(&chars, index) {
                if depth == 0 {
                    flags.extend(chars[index..=end].iter());
                }
                index = end + 1;
                continue;
            }
            depth += 1;
        } else if character == ')' && depth > 0 {
            depth -= 1;
        }
        index += 1;
    }
    flags
}

fn persistent_flag(chars: &[char], start: usize) -> Option<usize> {
    if chars.get(start + 1) != Some(&'?') {
        return None;
    }
    let mut index = start + 2;
    let mut dash = false;
    let mut flag = false;
    while let Some(character) = chars.get(index).copied() {
        match character {
            '-' if !dash => dash = true,
            'i' | 'm' | 's' | 'u' | 'U' | 'x' | 'R' => flag = true,
            ')' if flag => return Some(index),
            _ => return None,
        }
        index += 1;
    }
    None
}

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
