pub(super) fn parts(cmd: &str) -> Vec<&str> {
    let chars: Vec<(usize, char)> = cmd.char_indices().collect();
    let mut parts = Vec::new();
    let mut start = 0;
    let mut quote = None;
    let mut index = 0;
    while index < chars.len() {
        let (at, ch) = chars[index];
        if let Some(open) = quote {
            if ch == open {
                quote = None;
            }
            index += 1;
            continue;
        }
        if ch == '\'' || ch == '"' {
            quote = Some(ch);
            index += 1;
            continue;
        }
        let width = operator_width(&chars, index);
        if width == 0 {
            index += 1;
            continue;
        }
        push_part(&mut parts, cmd, start, at);
        index += width;
        start = chars.get(index).map(|(next, _)| *next).unwrap_or(cmd.len());
    }
    push_part(&mut parts, cmd, start, cmd.len());
    parts
}

fn operator_width(chars: &[(usize, char)], index: usize) -> usize {
    match chars.get(index).map(|(_, ch)| *ch) {
        Some(';') => 1,
        Some('&') if next_is(chars, index, '&') => 2,
        Some('|') if next_is(chars, index, '|') => 2,
        Some('|') => 1,
        _ => 0,
    }
}

fn next_is(chars: &[(usize, char)], index: usize, expected: char) -> bool {
    chars.get(index + 1).is_some_and(|(_, ch)| *ch == expected)
}

fn push_part<'a>(parts: &mut Vec<&'a str>, cmd: &'a str, start: usize, end: usize) {
    let part = cmd[start..end].trim();
    if !part.is_empty() {
        parts.push(part);
    }
}

pub(super) fn heredoc_at(line: &str) -> Option<usize> {
    let chars: Vec<(usize, char)> = line.char_indices().collect();
    let mut arith = 0i32;
    let mut index = 0;
    while index < chars.len() {
        if starts(&chars, index, &['$', '(', '(']) {
            arith += 1;
            index += 3;
            continue;
        }
        if arith > 0 && starts(&chars, index, &[')', ')']) {
            arith -= 1;
            index += 2;
            continue;
        }
        if starts(&chars, index, &['<', '<', '<']) {
            index += 3;
            continue;
        }
        if arith == 0 && starts(&chars, index, &['<', '<']) {
            return Some(chars[index].0);
        }
        index += if starts(&chars, index, &['<', '<']) {
            2
        } else {
            1
        };
    }
    None
}

fn starts(chars: &[(usize, char)], index: usize, prefix: &[char]) -> bool {
    prefix.iter().enumerate().all(|(offset, expected)| {
        chars
            .get(index + offset)
            .is_some_and(|(_, ch)| ch == expected)
    })
}
