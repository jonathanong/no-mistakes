use crate::codebase::shell_words::{split_quoted_words, Whitespace};

pub(super) fn segments(script: &str) -> Vec<&str> {
    let mut found = Vec::new();
    let mut start = 0;
    let mut chars = script.char_indices().peekable();
    let mut quote = None;
    let mut arithmetic_parens: Option<usize> = None;
    let mut token_start = true;
    let mut heredocs = Vec::new();
    while let Some((offset, character)) = chars.next() {
        if character == '\\' && quote != Some('\'') {
            chars.next();
            token_start = false;
            continue;
        }
        if let Some(open) = quote {
            if open == character {
                quote = None;
            }
            continue;
        }
        match character {
            '\'' | '"' | '`' => {
                quote = Some(character);
                token_start = false;
            }
            '(' => {
                if let Some(depth) = arithmetic_parens.as_mut() {
                    *depth += 1;
                } else if chars.next_if(|(_, next)| *next == '(').is_some() {
                    arithmetic_parens = Some(0);
                }
                token_start = false;
            }
            ')' => {
                if let Some(depth) = arithmetic_parens.as_mut() {
                    if *depth > 0 {
                        *depth -= 1;
                    } else {
                        chars.next_if(|(_, next)| *next == ')');
                        arithmetic_parens = None;
                    }
                }
                token_start = false;
            }
            '<' if arithmetic_parens.is_none()
                && script[offset..].starts_with("<<")
                && !script[offset..].starts_with("<<<")
                && !script[..offset].ends_with('<') =>
            {
                if let Some(heredoc) = HereDoc::parse(&script[offset + 2..]) {
                    heredocs.push(heredoc);
                }
                token_start = false;
            }
            '#' if token_start => {
                found.push(&script[start..offset]);
                start = next_line(script, &mut chars, &mut heredocs);
            }
            '\n' => {
                found.push(&script[start..offset]);
                start = skip_heredocs(script, &mut chars, &mut heredocs, offset + 1);
                token_start = true;
            }
            ';' => {
                found.push(&script[start..offset]);
                start = offset + 1;
                token_start = true;
            }
            '|' if arithmetic_parens.is_none() && !script[..offset].ends_with('>') => {
                found.push(&script[start..offset]);
                start = chars
                    .next_if(|(_, next)| matches!(next, '|' | '&'))
                    .map_or(offset + 1, |(next, _)| next + 1);
                token_start = true;
            }
            '&' if arithmetic_parens.is_none()
                && chars.next_if(|(_, next)| *next == '&').is_some() => {
                found.push(&script[start..offset]);
                start = offset + 2;
                token_start = true;
            }
            _ => token_start = character.is_ascii_whitespace(),
        }
    }
    found.push(&script[start..]);
    found
}

pub(super) fn expands_any(segment: &str, variables: &[String]) -> bool {
    if variables.is_empty() {
        return false;
    }
    let mut chars = segment.char_indices();
    let mut quote = None;
    while let Some((offset, ch)) = chars.next() {
        match ch {
            '\\' if quote != Some('\'') => {
                chars.next();
            }
            '\'' | '"' if quote.is_none() => quote = Some(ch),
            ch if quote == Some(ch) => quote = None,
            '$' if quote != Some('\'') => {
                let tail = &segment[offset + 1..];
                if tail.starts_with('$') {
                    chars.next();
                    continue;
                }
                let name = tail.strip_prefix('{').unwrap_or(tail);
                let end = name
                    .find(|c: char| !c.is_ascii_alphanumeric() && c != '_')
                    .unwrap_or(name.len());
                if variables.iter().any(|variable| variable == &name[..end]) {
                    return true;
                }
            }
            _ => {}
        }
    }
    false
}

struct HereDoc {
    delimiter: String,
    strip_tabs: bool,
}

impl HereDoc {
    fn parse(text: &str) -> Option<Self> {
        let strip_tabs = text.starts_with('-');
        let text = text
            .strip_prefix('-')
            .unwrap_or(text)
            .trim_start_matches([' ', '\t']);
        let end = match text.as_bytes().first()? {
            quote @ (b'\'' | b'"') => {
                text.as_bytes()[1..]
                    .iter()
                    .position(|character| character == quote)?
                    + 2
            }
            _ => text
                .find(|c: char| c.is_ascii_whitespace() || ";&|()<>".contains(c))
                .unwrap_or(text.len()),
        };
        let words = split_quoted_words(&text[..end], Whitespace::Ascii)?;
        let [delimiter] = words.try_into().ok()?;
        Some(Self {
            delimiter,
            strip_tabs,
        })
    }

    fn ends_at(&self, line: &str) -> bool {
        let line = if self.strip_tabs {
            line.trim_start_matches('\t')
        } else {
            line
        };
        line == self.delimiter
    }
}

fn next_line(
    script: &str,
    chars: &mut impl Iterator<Item = (usize, char)>,
    heredocs: &mut Vec<HereDoc>,
) -> usize {
    let start = chars
        .find_map(|(offset, c)| (c == '\n').then_some(offset + 1))
        .unwrap_or(script.len());
    skip_heredocs(script, chars, heredocs, start)
}

fn skip_heredocs(
    script: &str,
    chars: &mut impl Iterator<Item = (usize, char)>,
    heredocs: &mut Vec<HereDoc>,
    mut start: usize,
) -> usize {
    for heredoc in heredocs.drain(..) {
        loop {
            let Some(end) = chars.find_map(|(offset, c)| (c == '\n').then_some(offset)) else {
                return script.len();
            };
            let line = &script[start..end];
            start = end + 1;
            if heredoc.ends_at(line) {
                break;
            }
        }
    }
    start
}
