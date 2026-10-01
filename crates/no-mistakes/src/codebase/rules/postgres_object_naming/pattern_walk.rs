use super::pattern_flags::{apply_verbose, flag_span};

pub(super) fn active_flags(pre: &str) -> String {
    let chars: Vec<char> = pre.chars().collect();
    let mut flags = String::new();
    let mut depth = 0i32;
    let mut index = 0;
    let mut in_class = false;
    let mut verbose = false;
    let mut comment = false;
    let mut scopes: Vec<bool> = Vec::new();
    while index < chars.len() {
        let character = chars[index];
        if comment {
            if character == '\n' {
                comment = false;
            }
            index += 1;
            continue;
        }
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
        if character == '#' && verbose {
            comment = true;
            index += 1;
            continue;
        }
        if character == '(' {
            if let Some(end) = persistent_flag(&chars, index) {
                let body: String = chars[index + 2..end].iter().collect();
                apply_verbose(&body, &mut verbose);
                if depth == 0 {
                    flags.extend(chars[index..=end].iter());
                }
                index = end + 1;
                continue;
            }
            depth += 1;
            scopes.push(verbose);
        } else if character == ')' && depth > 0 {
            depth -= 1;
            if let Some(previous) = scopes.pop() {
                verbose = previous;
            }
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

pub(super) struct PlaceholderWalk {
    pub offsets: Vec<usize>,
    pub plain: bool,
}

pub(super) fn walk_placeholders(pattern: &str) -> PlaceholderWalk {
    let chars: Vec<(usize, char)> = pattern.char_indices().collect();
    let mut index = 0;
    let mut depth = 0i32;
    let mut in_class = false;
    let mut top_alt = false;
    let mut verbose = false;
    let mut comment = false;
    let mut ok = false;
    let mut bad = false;
    let mut offsets = Vec::new();
    let mut scopes: Vec<bool> = Vec::new();
    while index < chars.len() {
        let (byte, character) = chars[index];
        if comment {
            if character == '\n' {
                comment = false;
            }
            index += 1;
            continue;
        }
        if character == '\\' {
            index += 2;
            continue;
        }
        if in_class {
            if pattern[byte..].starts_with("{table}") {
                offsets.push(byte);
                bad = true;
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
        if character == '#' && verbose {
            comment = true;
            index += 1;
            continue;
        }
        if character == '(' {
            if let Some(span) = flag_span(&chars, index) {
                let body: String = chars[index + 2..span.end]
                    .iter()
                    .map(|(_, char)| char)
                    .collect();
                if span.scoped {
                    scopes.push(verbose);
                    depth += 1;
                }
                apply_verbose(&body, &mut verbose);
                index = span.end + 1;
                continue;
            }
            depth += 1;
            scopes.push(verbose);
        }
        if character == ')' && depth > 0 {
            depth -= 1;
            if let Some(previous) = scopes.pop() {
                verbose = previous;
            }
        }
        if character == '|' && depth == 0 {
            top_alt = true;
        }
        if pattern[byte..].starts_with("{table}") {
            offsets.push(byte);
            if depth != 0 {
                bad = true;
            }
            ok = true;
        }
        index += 1;
    }
    PlaceholderWalk {
        offsets,
        plain: ok && !top_alt && !bad,
    }
}

pub(super) fn placeholder_is_plain(pattern: &str) -> bool {
    walk_placeholders(pattern).plain
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
