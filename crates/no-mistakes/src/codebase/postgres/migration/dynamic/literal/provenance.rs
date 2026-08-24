use super::super::*;

#[derive(Clone, Copy)]
enum LiteralEscape {
    Raw,
    Plain,
    Escaped,
    Unicode(char),
}

pub(super) fn literal_source_bytes(
    source: &str,
    token: &TokenWithSpan,
    decoded: &str,
    unicode_escape: Option<char>,
) -> Option<Vec<usize>> {
    let start = location_offset(source, token.span.start.line, token.span.start.column)?;
    let end = location_offset(source, token.span.end.line, token.span.end.column)?;
    let raw = source.get(start..end)?;
    let (content, escape) = match &token.token {
        Token::SingleQuotedString(_) => {
            (raw.get(1..raw.len().checked_sub(1)?)?, LiteralEscape::Plain)
        }
        Token::EscapedStringLiteral(_) => (
            raw.get(2..raw.len().checked_sub(1)?)?,
            LiteralEscape::Escaped,
        ),
        Token::UnicodeStringLiteral(_) => (
            raw.get(3..raw.len().checked_sub(1)?)?,
            LiteralEscape::Unicode(unicode_escape.unwrap_or('\\')),
        ),
        Token::DollarQuotedString(value) => {
            let delimiter_len = value.tag.as_ref().map_or(2, |tag| tag.len() + 2);
            (
                raw.get(delimiter_len..raw.len().checked_sub(delimiter_len)?)?,
                LiteralEscape::Raw,
            )
        }
        _ => return None,
    };
    map_decoded_source_bytes(content, decoded, escape, token.span.start.line as usize)
}

fn map_decoded_source_bytes(
    raw: &str,
    decoded: &str,
    escape: LiteralEscape,
    mut source_line: usize,
) -> Option<Vec<usize>> {
    let raw = raw.chars().collect::<Vec<_>>();
    let mut at = 0usize;
    let mut source_bytes = Vec::with_capacity(decoded.len());
    for character in decoded.chars() {
        let doubled_quote = raw.get(at) == Some(&'\'') && raw.get(at + 1) == Some(&'\'');
        let width = match escape {
            LiteralEscape::Raw => 1,
            LiteralEscape::Plain | LiteralEscape::Escaped | LiteralEscape::Unicode(_)
                if doubled_quote =>
            {
                2
            }
            LiteralEscape::Plain => 1,
            LiteralEscape::Escaped if raw.get(at) == Some(&'\\') => escaped_width(&raw, at),
            LiteralEscape::Escaped => 1,
            LiteralEscape::Unicode(marker) if raw.get(at) == Some(&marker) => {
                unicode_escape_width(&raw, at, marker)
            }
            LiteralEscape::Unicode(_) => 1,
        };
        let consumed = raw.get(at..at.checked_add(width)?)?;
        source_bytes.extend(std::iter::repeat_n(source_line, character.len_utf8()));
        source_line += consumed
            .iter()
            .filter(|character| **character == '\n')
            .count();
        at += width;
    }
    (at == raw.len()).then_some(source_bytes)
}

fn escaped_width(raw: &[char], at: usize) -> usize {
    match raw.get(at + 1).copied() {
        Some('u') => 6,
        Some('U') => 10,
        Some('x') => {
            2 + raw[at + 2..]
                .iter()
                .take(2)
                .take_while(|character| character.is_ascii_hexdigit())
                .count()
        }
        Some(character) if character.is_digit(8) => {
            1 + raw[at + 1..]
                .iter()
                .take(3)
                .take_while(|character| character.is_digit(8))
                .count()
        }
        Some(_) => 2,
        None => 1,
    }
}

fn unicode_escape_width(raw: &[char], at: usize, escape: char) -> usize {
    if raw.get(at + 1) == Some(&escape) {
        return 2;
    }
    if raw.get(at + 1) == Some(&'+') {
        return 8;
    }
    let width = 5;
    let high_surrogate = raw
        .get(at + 1..at + 5)
        .and_then(|digits| u32::from_str_radix(&digits.iter().collect::<String>(), 16).ok())
        .is_some_and(|value| (0xD800..=0xDBFF).contains(&value));
    if high_surrogate && raw.get(at + width) == Some(&escape) {
        width + 5
    } else {
        width
    }
}
