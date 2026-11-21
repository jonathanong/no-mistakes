pub(super) fn stdout_leaves(line: &str) -> bool {
    let chars: Vec<(usize, char)> = line.char_indices().collect();
    let mut quote = None;
    let mut arith = 0i32;
    let mut index = 0;
    while index < chars.len() {
        let (_, ch) = chars[index];
        if let Some(open) = quote {
            if ch == open {
                quote = None;
            }
            index += 1;
            continue;
        }
        if ch == '"' || ch == '\'' {
            quote = Some(ch);
            index += 1;
            continue;
        }
        if super::body::starts(&chars, index, &['$', '(', '(']) {
            arith += 1;
            index += 3;
            continue;
        }
        if arith > 0 && super::body::starts(&chars, index, &[')', ')']) {
            arith -= 1;
            index += 2;
            continue;
        }
        if arith > 0 {
            index += 1;
            continue;
        }
        if ch == '>' && index > 0 && chars[index - 1].1 == '>' {
            index += 1;
            continue;
        }
        if ch == '>' && chars.get(index + 1).is_some_and(|(_, next)| *next == '=') {
            index += 2;
            continue;
        }
        if ch == '>' && fd_before(&chars, index) == 1 && dup_leaves(&chars, index) {
            return true;
        }
        index += 1;
    }
    false
}

fn dup_leaves(chars: &[(usize, char)], gt: usize) -> bool {
    if !chars.get(gt + 1).is_some_and(|(_, ch)| *ch == '&') {
        return true;
    }
    fd_number(chars, gt + 2) != Some(1)
}

fn fd_before(chars: &[(usize, char)], gt: usize) -> u32 {
    let mut index = gt;
    while index > 0 && chars[index - 1].1.is_ascii_digit() {
        index -= 1;
    }
    if index == gt {
        return 1;
    }
    let text: String = chars[index..gt].iter().map(|(_, ch)| *ch).collect();
    text.parse().unwrap_or(1)
}

fn fd_number(chars: &[(usize, char)], mut at: usize) -> Option<u32> {
    while chars.get(at).is_some_and(|(_, ch)| ch.is_whitespace()) {
        at += 1;
    }
    let start = at;
    if !chars.get(at).is_some_and(|(_, ch)| ch.is_ascii_digit()) {
        return None;
    }
    while chars.get(at).is_some_and(|(_, ch)| ch.is_ascii_digit()) {
        at += 1;
    }
    if chars.get(at).is_some_and(|(_, ch)| !boundary(*ch)) {
        return None;
    }
    let text: String = chars[start..at].iter().map(|(_, ch)| *ch).collect();
    text.parse().ok()
}

fn boundary(ch: char) -> bool {
    ch.is_whitespace() || matches!(ch, '>' | '<' | '&' | '|' | ';' | '(' | ')')
}
