#[path = "shell_redirect.rs"]
mod redirect;

pub(super) use redirect::{mask_output_redirects, redirect_targets};

struct Piece<'a> {
    text: &'a str,
    piped: bool,
}

pub(super) fn parts(cmd: &str) -> Vec<&str> {
    pieces(cmd).into_iter().map(|piece| piece.text).collect()
}

pub(super) fn inherits_redirect(cmd: &str) -> Vec<(&str, bool)> {
    pieces(cmd)
        .into_iter()
        .map(|piece| (piece.text, !piece.piped))
        .collect()
}

fn pieces(cmd: &str) -> Vec<Piece<'_>> {
    let chars: Vec<(usize, char)> = cmd.char_indices().collect();
    let mut pieces = Vec::new();
    let mut start = 0;
    let mut quote = None;
    let mut arith = 0i32;
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
        if arith > 0 && ch == '&' {
            index += 1;
            continue;
        }
        let width = operator_width(&chars, index);
        if width == 0 {
            index += 1;
            continue;
        }
        push_piece(&mut pieces, cmd, start, at, ch == '|' && width == 1);
        index += width;
        start = chars.get(index).map(|(next, _)| *next).unwrap_or(cmd.len());
    }
    push_piece(&mut pieces, cmd, start, cmd.len(), false);
    pieces
}

fn operator_width(chars: &[(usize, char)], index: usize) -> usize {
    match chars.get(index).map(|(_, ch)| *ch) {
        Some(';') => 1,
        Some('&') if next_is(chars, index, '&') => 2,
        Some('&') if redirect_mark(chars, index) => 0,
        Some('&') => 1,
        Some('|') if next_is(chars, index, '|') => 2,
        Some('|') => 1,
        _ => 0,
    }
}

fn redirect_mark(chars: &[(usize, char)], index: usize) -> bool {
    if next_is(chars, index, '>') {
        return true;
    }
    let Some(prev) = index.checked_sub(1) else {
        return false;
    };
    matches!(chars.get(prev).map(|(_, ch)| *ch), Some('>' | '<'))
}

fn next_is(chars: &[(usize, char)], index: usize, expected: char) -> bool {
    chars.get(index + 1).is_some_and(|(_, ch)| *ch == expected)
}

fn push_piece<'a>(
    pieces: &mut Vec<Piece<'a>>,
    cmd: &'a str,
    start: usize,
    end: usize,
    piped: bool,
) {
    let text = cmd[start..end].trim();
    if !text.is_empty() {
        pieces.push(Piece { text, piped });
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
