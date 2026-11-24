use std::iter::Peekable;
use std::str::CharIndices;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum SlashKind {
    Comment,
    Regex,
}

pub(super) fn skip_slash(
    chars: &mut Peekable<CharIndices<'_>>,
    allows_regex: bool,
) -> Option<Option<SlashKind>> {
    match chars.peek().map(|(_, next)| *next) {
        Some('/') => {
            chars.next();
            for (_, next) in chars.by_ref() {
                if next == '\n' {
                    break;
                }
            }
            Some(Some(SlashKind::Comment))
        }
        Some('*') => {
            chars.next();
            while let Some((_, next)) = chars.next() {
                if next == '*' && chars.next_if(|(_, next)| *next == '/').is_some() {
                    return Some(Some(SlashKind::Comment));
                }
            }
            None
        }
        _ if allows_regex => {
            let mut probe = chars.clone();
            if regex_end(&mut probe) {
                *chars = probe;
                Some(Some(SlashKind::Regex))
            } else {
                Some(None)
            }
        }
        _ => Some(None),
    }
}

fn regex_end(chars: &mut Peekable<CharIndices<'_>>) -> bool {
    let mut in_class = false;
    while let Some((_, ch)) = chars.next() {
        if ch == '\\' {
            if chars
                .next()
                .is_none_or(|(_, esc)| matches!(esc, '\n' | '\r'))
            {
                return false;
            }
            continue;
        }
        if matches!(ch, '\n' | '\r') {
            return false;
        }
        if ch == '[' {
            in_class = true;
        } else if ch == ']' {
            in_class = false;
        } else if ch == '/' && !in_class {
            return true;
        }
    }
    false
}
