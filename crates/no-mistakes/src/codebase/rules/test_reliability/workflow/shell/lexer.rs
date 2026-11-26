use crate::codebase::shell_heredoc::{ascii_document, skip_body, CarriageReturn};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Connector {
    None,
    And,
    Or,
    Pipe,
}

pub(super) struct Segment<'a> {
    pub(super) text: &'a str,
    pub(super) preceded_by: Connector,
}

pub(super) fn segments(script: &str) -> Vec<Segment<'_>> {
    let mut found = Vec::new();
    let mut start = 0;
    let mut base = 0;
    let mut preceded_by = Connector::None;
    let mut chars = script.char_indices().peekable();
    let mut quote = None;
    let mut arithmetic_parens: Option<usize> = None;
    let mut token_start = true;
    let mut prev_gt = false;
    let mut heredocs = Vec::new();
    while let Some((relative, character)) = chars.next() {
        let offset = base + relative;
        let after_gt = std::mem::replace(&mut prev_gt, false);
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
            '>' => {
                prev_gt = true;
                token_start = false;
            }
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
            '<' if !script[..offset].ends_with('<') => {
                if let Some(document) =
                    ascii_document(&script[offset..], arithmetic_parens.is_some())
                {
                    heredocs.push(document);
                }
                token_start = false;
            }
            '#' | '\n' if character == '\n' || token_start => {
                let text = &script[start..offset];
                found.push(Segment { text, preceded_by });
                if text.bytes().any(|b| !b.is_ascii_whitespace()) {
                    preceded_by = Connector::None;
                }
                let body_start = script[offset..]
                    .find('\n')
                    .map_or(script.len(), |newline| offset + newline + 1);
                start = heredocs.drain(..).fold(body_start, |start, document| {
                    skip_body(
                        script.as_bytes(),
                        start,
                        document.delimiter.bytes(),
                        document.strip_tabs,
                        CarriageReturn::Keep,
                    )
                    .end
                });
                base = start;
                chars = script[base..].char_indices().peekable();
                token_start = true;
            }
            ';' => {
                found.push(Segment {
                    text: &script[start..offset],
                    preceded_by,
                });
                preceded_by = Connector::None;
                start = offset + 1;
                token_start = true;
            }
            '|' if arithmetic_parens.is_none() && !after_gt => {
                found.push(Segment {
                    text: &script[start..offset],
                    preceded_by,
                });
                match chars.next_if(|(_, next)| matches!(next, '|' | '&')) {
                    Some((next, '|')) => {
                        preceded_by = Connector::Or;
                        start = base + next + 1;
                    }
                    Some((next, _)) => {
                        preceded_by = Connector::Pipe;
                        start = base + next + 1;
                    }
                    None => {
                        preceded_by = Connector::Pipe;
                        start = offset + 1;
                    }
                }
                token_start = true;
            }
            '&' if arithmetic_parens.is_none()
                && chars.next_if(|(_, next)| *next == '&').is_some() =>
            {
                found.push(Segment {
                    text: &script[start..offset],
                    preceded_by,
                });
                preceded_by = Connector::And;
                start = offset + 2;
                token_start = true;
            }
            _ => token_start = character.is_ascii_whitespace(),
        }
    }
    found.push(Segment {
        text: &script[start..],
        preceded_by,
    });
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
