pub(super) struct Opener {
    delim: String,
    strip_tabs: bool,
}

pub(super) enum Look {
    Herestring { index: usize },
    Document { index: usize, opener: Opener },
}

pub(super) struct Body {
    pub(super) index: usize,
    pub(super) line: usize,
    pub(super) terminated: bool,
}

pub(super) fn look(chars: &[char], index: usize) -> Option<Look> {
    if chars.get(index) != Some(&'<') || chars.get(index + 1) != Some(&'<') {
        return None;
    }
    if chars.get(index + 2) == Some(&'<') {
        return Some(Look::Herestring { index: index + 3 });
    }
    let mut next = index + 2;
    let strip_tabs = chars.get(next) == Some(&'-');
    if strip_tabs {
        next += 1;
    }
    let (delim, next) = delim(chars, next);
    Some(Look::Document {
        index: next,
        opener: Opener { delim, strip_tabs },
    })
}

pub(super) fn skip_body(chars: &[char], index: usize, line: usize, opener: &Opener) -> Body {
    let mut index = index;
    let mut line = line;
    if chars.get(index) == Some(&'\n') {
        index += 1;
        line += 1;
    }
    loop {
        if index >= chars.len() {
            return Body {
                index,
                line,
                terminated: false,
            };
        }
        let start = index;
        while index < chars.len() && chars[index] != '\n' {
            index += 1;
        }
        let text: String = chars[start..index].iter().collect();
        let compare = if opener.strip_tabs {
            text.trim_start_matches('\t')
        } else {
            text.as_str()
        };
        let done = compare == opener.delim;
        if index < chars.len() {
            index += 1;
            line += 1;
        }
        if done {
            return Body {
                index,
                line,
                terminated: true,
            };
        }
    }
}

fn delim(chars: &[char], mut index: usize) -> (String, usize) {
    while matches!(chars.get(index), Some(' ' | '\t')) {
        index += 1;
    }
    let quote = match chars.get(index) {
        Some(ch @ ('\'' | '"')) => {
            index += 1;
            Some(*ch)
        }
        _ => None,
    };
    let mut text = String::new();
    while let Some(ch) = chars.get(index).copied() {
        if quote == Some(ch) {
            index += 1;
            break;
        }
        if quote.is_none() && (ch.is_whitespace() || ch == ';') {
            break;
        }
        text.push(ch);
        index += 1;
    }
    (text, index)
}
