/// Shell syntax inside `$(...)` where an apostrophe is literal text.
///
/// Comments, here-document bodies and `$'...'` strings never open a quote
/// frame.
pub(super) struct Heredoc {
    delimiter: String,
    strip_tabs: bool,
}

pub(super) fn is_comment_start(chars: &[char], index: usize) -> bool {
    chars[index] == '#'
        && index > 0
        && matches!(chars[index - 1], ' ' | '\t' | '\n' | '(' | ';' | '&' | '|')
}

pub(super) fn take_comment(chars: &[char], index: &mut usize, word: &mut String) {
    while *index < chars.len() && chars[*index] != '\n' {
        word.push(chars[*index]);
        *index += 1;
    }
}

pub(super) fn take_heredoc_start(
    chars: &[char],
    index: &mut usize,
    word: &mut String,
) -> Option<Heredoc> {
    if chars[*index] != '<'
        || chars.get(*index + 1) != Some(&'<')
        || chars.get(*index + 2) == Some(&'<')
        || (*index > 0 && chars[*index - 1] == '<')
    {
        return None;
    }
    let mut at = *index + 2;
    let strip_tabs = chars.get(at) == Some(&'-');
    if strip_tabs {
        at += 1;
    }
    while matches!(chars.get(at), Some(' ' | '\t')) {
        at += 1;
    }
    let quote = chars.get(at).copied().filter(|ch| matches!(ch, '\'' | '"'));
    let start = at + usize::from(quote.is_some());
    let first = *chars.get(start)?;
    if !(first.is_ascii_alphabetic() || first == '_') {
        return None;
    }
    let mut end = start;
    while end < chars.len()
        && match quote {
            Some(q) => chars[end] != q,
            None => chars[end].is_ascii_alphanumeric() || chars[end] == '_',
        }
    {
        end += 1;
    }
    let delimiter: String = chars[start..end].iter().collect();
    let stop = end + usize::from(quote.is_some() && end < chars.len());
    word.extend(&chars[*index..stop]);
    *index = stop;
    Some(Heredoc {
        delimiter,
        strip_tabs,
    })
}

/// Called at the newline ending the line that holds the here-document
/// operators. The caller already counted that newline, so it is uncounted here
/// and every newline pushed below is counted once.
pub(super) fn take_heredoc_bodies(
    chars: &[char],
    index: &mut usize,
    word: &mut String,
    line: &mut usize,
    pending: &mut Vec<Heredoc>,
) {
    *line -= 1;
    for doc in pending.drain(..) {
        while *index < chars.len() {
            word.push('\n');
            *line += 1;
            *index += 1;
            let start = *index;
            while *index < chars.len() && chars[*index] != '\n' {
                *index += 1;
            }
            let text: String = chars[start..*index].iter().collect();
            word.push_str(&text);
            let body = if doc.strip_tabs {
                text.trim_start_matches('\t')
            } else {
                text.as_str()
            };
            if body == doc.delimiter {
                break;
            }
        }
    }
}
