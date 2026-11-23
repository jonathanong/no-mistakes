#[path = "shell_fd.rs"]
mod fd;
#[path = "shell_heredoc.rs"]
mod heredoc;
#[path = "shell_redirect.rs"]
mod redirect;

pub(super) use fd::redirects_stdout;
pub(super) use heredoc::heredoc_at;
pub(super) use redirect::{mask_output_redirects, redirect_targets};

struct Piece<'a> {
    text: &'a str,
    piped: bool,
    origin: usize,
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

pub(super) fn split_head(cmd: &str) -> (&str, &str) {
    let parts = pieces(cmd);
    let Some(first) = parts.first() else {
        return ("", "");
    };
    if cmd[..first.origin].contains([';', '&', '|']) {
        return ("", cmd[first.origin..].trim());
    }
    if let Some(next) = parts.get(1) {
        return (first.text, cmd[next.origin..].trim());
    }
    (first.text, "")
}

pub(super) fn piped_head(cmd: &str) -> Option<&str> {
    let piece = pieces(cmd).into_iter().next()?;
    piece.piped.then_some(piece.text)
}

fn pieces(cmd: &str) -> Vec<Piece<'_>> {
    let chars: Vec<(usize, char)> = cmd.char_indices().collect();
    let mut pieces = Vec::new();
    let mut start = 0;
    let mut quote = None;
    let mut arith = 0i32;
    let mut subst = 0i32;
    let mut braces = 0i32;
    let mut parens = 0i32;
    let mut tick = false;
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
        if ch == '`' {
            tick = !tick;
            index += 1;
            continue;
        }
        if tick {
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
        if arith > 0 {
            index += 1;
            continue;
        }
        if starts(&chars, index, &['$', '(']) {
            subst += 1;
            index += 2;
            continue;
        }
        if subst > 0 {
            if ch == ')' {
                subst -= 1;
            }
            index += 1;
            continue;
        }
        let width = if braces == 0 && parens == 0 {
            operator_width(&chars, index)
        } else {
            0
        };
        if width == 0 {
            match ch {
                '{' => braces += 1,
                '}' if braces > 0 => braces -= 1,
                '(' => parens += 1,
                ')' if parens > 0 => parens -= 1,
                _ => {}
            }
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
    let raw = &cmd[start..end];
    let text = raw.trim();
    if text.is_empty() {
        return;
    }
    let leading = raw.len() - raw.trim_start().len();
    pieces.push(Piece {
        text,
        piped,
        origin: start + leading,
    });
}

fn starts(chars: &[(usize, char)], index: usize, prefix: &[char]) -> bool {
    prefix.iter().enumerate().all(|(offset, expected)| {
        chars
            .get(index + offset)
            .is_some_and(|(_, ch)| ch == expected)
    })
}

fn is_meta(ch: char) -> bool {
    matches!(ch, '>' | '<' | '|' | '&' | ';' | '(' | ')')
}
