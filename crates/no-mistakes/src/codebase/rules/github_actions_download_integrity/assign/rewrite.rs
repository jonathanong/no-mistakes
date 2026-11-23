use super::super::words::is_assignment;
use super::{probe, reference_name, Bindings};
use std::ops::Range;

#[derive(Default)]
pub(crate) struct Script {
    pub(crate) text: String,
    origins: Vec<Option<usize>>,
    quote: Option<char>,
    in_word: bool,
}

impl Script {
    pub(crate) fn original_anchor(&self, anchor: usize) -> Option<usize> {
        match anchor.checked_sub(1) {
            Some(separator) => self.origins[separator].map(|raw| raw + 1),
            None => Some(0),
        }
    }

    fn source_char(&mut self, ch: char, offset: usize) {
        self.text.push(ch);
        self.origins.push(Some(offset));
    }

    pub(super) fn substitute<'a>(
        &mut self,
        line: &str,
        offset: usize,
        heredocs: &mut &[Range<usize>],
        lookup: impl Fn(&str) -> Option<&'a str>,
    ) {
        let mut chars = line.chars().enumerate().peekable();
        while let Some((index, ch)) = chars.next() {
            let raw = offset + index;
            while heredocs.first().is_some_and(|range| range.end <= raw) {
                *heredocs = &heredocs[1..];
            }
            if let Some(range) = heredocs.first().filter(|range| range.contains(&raw)) {
                self.source_char(ch, raw);
                while let Some((index, ch)) = chars.next_if(|(index, _)| offset + index < range.end)
                {
                    self.source_char(ch, offset + index);
                }
                self.quote = None;
                self.in_word = true;
                continue;
            }
            if ch == '#' && self.quote.is_none() && !self.in_word {
                self.source_char(ch, offset + index);
                for (index, ch) in chars {
                    self.source_char(ch, offset + index);
                }
                return;
            }
            if ch == '\\' && self.quote != Some('\'') {
                self.source_char(ch, offset + index);
                let Some((escaped_index, escaped)) = chars.next() else {
                    return;
                };
                self.source_char(escaped, offset + escaped_index);
                self.in_word = true;
                continue;
            }
            if ch == '$' && self.quote != Some('\'') {
                self.in_word = true;
                let mut reference = String::new();
                if matches!(chars.peek(), Some((_, '{'))) {
                    for (_, next) in chars.by_ref() {
                        reference.push(next);
                        if next == '}' {
                            break;
                        }
                    }
                } else {
                    while let Some((_, next)) =
                        chars.next_if(|(_, next)| next.is_ascii_alphanumeric() || *next == '_')
                    {
                        reference.push(next);
                    }
                }
                let value = lookup(reference_name(&reference))
                    .filter(|value| preserves_shell_syntax(value, self.quote));
                if let Some(value) = value {
                    self.text.push_str(value);
                    self.origins.extend(value.chars().map(|_| None));
                } else {
                    self.source_char(ch, offset + index);
                    for (relative, ch) in reference.chars().enumerate() {
                        self.source_char(ch, offset + index + relative + 1);
                    }
                }
                continue;
            }
            if matches!(ch, '\'' | '"') {
                if self.quote == Some(ch) {
                    self.quote = None;
                } else if self.quote.is_none() {
                    self.quote = Some(ch);
                }
            }
            self.in_word = self.quote.is_some() || (!ch.is_whitespace() && !";&|()<>".contains(ch));
            self.source_char(ch, offset + index);
        }
        self.in_word = self.quote.is_some();
    }
}

fn preserves_shell_syntax(value: &str, quote: Option<char>) -> bool {
    !(quote.is_some() && value.is_empty())
        && value.split_whitespace().all(|field| {
            !is_assignment(field)
                && !matches!(
                    field,
                    "!" | "if" | "elif" | "then" | "else" | "do" | "while" | "until" | "time"
                )
        })
        && value.chars().all(|ch| {
            (ch == '\t' || !ch.is_control())
                && (!ch.is_whitespace() || matches!(ch, ' ' | '\t'))
                && !matches!(
                    ch,
                    '\'' | '"'
                        | '\\'
                        | '`'
                        | '$'
                        | '#'
                        | ';'
                        | '&'
                        | '|'
                        | '<'
                        | '>'
                        | '('
                        | ')'
                        | '{'
                        | '}'
                )
                && (quote.is_some() || !matches!(ch, '*' | '?' | '['))
        })
}

pub(in super::super) fn prepare_max_script(
    script: &str,
    bindings: &Bindings,
    mut heredocs: &[Range<usize>],
) -> Option<Script> {
    probe::note();
    let has_literals = bindings.has_literals();
    if !has_literals && !script.contains("\r\n") && (script.is_empty() || script.ends_with('\n')) {
        return None;
    }
    let mut out = Script::default();
    let mut offset = 0;
    for (index, source) in script.split_inclusive('\n').enumerate() {
        let line = source
            .strip_suffix('\n')
            .map_or(source, |body| body.strip_suffix('\r').unwrap_or(body));
        if has_literals {
            out.substitute(line, offset, &mut heredocs, |name| {
                bindings.literal(name, index + 1)
            });
        } else {
            for (index, ch) in line.chars().enumerate() {
                out.source_char(ch, offset + index);
            }
        }
        offset += source.chars().count();
        out.text.push('\n');
        out.origins.push(source.ends_with('\n').then(|| offset - 1));
    }
    Some(out)
}
