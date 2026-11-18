use super::super::words::is_assignment;
use super::{classify, probe, single_literal_binding, Kind};

#[derive(Default)]
pub(crate) struct Script {
    pub(crate) text: String,
    origins: Vec<Option<usize>>,
    quote: Option<char>,
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

    pub(super) fn substitute(&mut self, line: &str, offset: usize, name: &str, value: &str) {
        let mut chars = line.chars().enumerate().peekable();
        while let Some((index, ch)) = chars.next() {
            if ch == '\\' && self.quote != Some('\'') {
                self.source_char(ch, offset + index);
                if let Some((escaped_index, escaped)) = chars.next() {
                    self.source_char(escaped, offset + escaped_index);
                }
                continue;
            }
            if ch == '$' && self.quote != Some('\'') {
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
                let matches_binding = reference == name
                    || reference
                        .strip_prefix('{')
                        .and_then(|text| text.strip_suffix('}'))
                        == Some(name);
                if matches_binding && preserves_shell_syntax(value, self.quote) {
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
            self.source_char(ch, offset + index);
        }
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

pub(crate) fn prepare_max_script(script: &str) -> Option<Script> {
    probe::note();
    let binding = single_literal_binding(script);
    if binding.is_none()
        && !script.contains("\r\n")
        && (script.is_empty() || script.ends_with('\n'))
    {
        return None;
    }
    let mut out = Script::default();
    let mut offset = 0;
    for source in script.split_inclusive('\n') {
        let line = source
            .strip_suffix('\n')
            .map_or(source, |body| body.strip_suffix('\r').unwrap_or(body));
        if let Some((name, value)) = binding
            .as_ref()
            .filter(|_| !matches!(classify(line.trim()), Kind::Literal(_, _)))
        {
            out.substitute(line, offset, name, value);
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
