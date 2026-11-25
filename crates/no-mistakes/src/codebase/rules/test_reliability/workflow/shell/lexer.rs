use crate::codebase::shell_heredoc::{ascii_document, skip_body, CarriageReturn};

pub(super) fn segments(script: &str) -> Vec<&str> {
    let mut found = Vec::new();
    let mut start = 0;
    let mut base = 0;
    let mut chars = script.char_indices().peekable();
    let mut quote = None;
    let mut arithmetic_parens: Option<usize> = None;
    let mut token_start = true;
    let mut heredocs = Vec::new();
    while let Some((relative, character)) = chars.next() {
        let offset = base + relative;
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
            '<' if !script[..offset].ends_with('<') => {
                if let Some(document) = ascii_document(&script[offset..], arithmetic_parens.is_some()) {
                    heredocs.push(document);
                }
                token_start = false;
            }
            '#' | '\n' if character == '\n' || token_start => {
                found.push(&script[start..offset]);
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
                found.push(&script[start..offset]);
                start = offset + 1;
                token_start = true;
            }
            '&' | '|' if chars.next_if(|(_, next)| *next == character).is_some() => {
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
