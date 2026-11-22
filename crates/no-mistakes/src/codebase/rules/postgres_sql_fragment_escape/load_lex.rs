use std::iter::Peekable;
use std::str::CharIndices;

pub(super) fn skip(chars: &mut Peekable<CharIndices<'_>>, ch: char) -> Option<bool> {
    if matches!(ch, '\'' | '"' | '`') {
        quoted_end(chars, ch, false)?;
        return Some(true);
    }
    if ch != '/' {
        return Some(false);
    }
    match chars.peek().map(|(_, next)| *next) {
        Some('/') => {
            chars.next();
            for (_, next) in chars.by_ref() {
                if next == '\n' {
                    break;
                }
            }
            Some(true)
        }
        Some('*') => {
            chars.next();
            while let Some((_, next)) = chars.next() {
                if next == '*' && chars.next_if(|(_, next)| *next == '/').is_some() {
                    return Some(true);
                }
            }
            None
        }
        _ => Some(false),
    }
}

fn quoted_end(chars: &mut Peekable<CharIndices<'_>>, quote: char, literal: bool) -> Option<usize> {
    while let Some((index, ch)) = chars.next() {
        if ch == '\u{5c}' {
            chars.next();
            continue;
        }
        if literal
            && quote == '`'
            && ch == '$'
            && chars.peek().is_some_and(|(_, next)| *next == '{')
        {
            return None;
        }
        if ch == quote {
            return Some(index + ch.len_utf8());
        }
    }
    None
}

pub(super) fn static_argument(text: &str) -> bool {
    let Some(text) = without_trivia(text) else {
        return false;
    };
    let mut chars = text.char_indices().peekable();
    let Some((_, quote @ ('\'' | '"' | '`'))) = chars.next() else {
        return false;
    };
    let Some(end) = quoted_end(&mut chars, quote, true) else {
        return false;
    };
    let Some(rest) = without_trivia(&text[end..]) else {
        return false;
    };
    rest.is_empty() || rest.starts_with([')', ',', '.'])
}

fn without_trivia(text: &str) -> Option<&str> {
    let mut chars = text.char_indices().peekable();
    while let Some((index, ch)) = chars.next() {
        if ch.is_whitespace() {
            continue;
        }
        if ch == '/' && skip(&mut chars, ch)? {
            continue;
        }
        return Some(&text[index..]);
    }
    Some("")
}
