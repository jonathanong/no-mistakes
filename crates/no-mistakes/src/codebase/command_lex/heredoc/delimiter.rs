use super::super::words::{split_quoted_words, Whitespace};
use super::{first_document, operator, Document, Operator};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum DelimiterStyle {
    StripQuotes,
    WhitespaceWord,
    AsciiQuotedWords,
}

struct Word {
    text: String,
    width: usize,
    quoted: bool,
    closed: bool,
}

pub(crate) fn char_delimiter(input: &[char], style: DelimiterStyle) -> (String, usize) {
    let start = input
        .iter()
        .take_while(|&&ch| matches!(ch, ' ' | '\t'))
        .count();
    let word = read_word(input[start..].iter().copied(), style);
    (word.text, start + word.width)
}

pub(crate) fn char_delimiter_quoted(input: &[char]) -> (String, usize, bool) {
    let start = input
        .iter()
        .take_while(|&&ch| matches!(ch, ' ' | '\t'))
        .count();
    let mut text = String::new();
    let mut index = start;
    let mut quote: Option<char> = None;
    let mut quoted = false;
    while index < input.len() {
        let ch = input[index];
        match quote {
            Some(open) if ch == open => {
                quote = None;
                index += 1;
            }
            Some(_) => {
                text.push(ch);
                index += 1;
            }
            None if matches!(ch, '\'' | '"') => {
                quote = Some(ch);
                quoted = true;
                index += 1;
            }
            None if ch == '\\' => {
                let Some(&next) = input.get(index + 1).filter(|&&c| c != '\n') else {
                    break;
                };
                text.push(next);
                quoted = true;
                index += 2;
            }
            None if ch.is_whitespace() || matches!(ch, ';' | '|' | '&' | '<' | '>' | '(' | ')') => {
                break;
            }
            None => {
                text.push(ch);
                index += 1;
            }
        }
    }
    (text, index, quoted && quote.is_none())
}

pub(crate) fn line_opener(line: &str) -> Option<(Document, bool)> {
    let at = first_document(line)?;
    let rest = line[at + 2..].trim_start();
    let (rest, strip_tabs) = rest
        .strip_prefix('-')
        .map_or((rest, false), |rest| (rest.trim_start(), true));
    let word = read_word(rest.chars(), DelimiterStyle::WhitespaceWord);
    (word.closed && (word.quoted || !word.text.is_empty())).then_some((
        Document {
            delimiter: word.text,
            strip_tabs,
        },
        word.quoted,
    ))
}

pub(crate) fn ascii_document(input: &str, arithmetic: bool) -> Option<Document> {
    let opener @ Operator::Document { strip_tabs } = operator(input.as_bytes(), arithmetic)? else {
        return None;
    };
    let text = input[opener.width()..].trim_start_matches([' ', '\t']);
    let word = read_word(text.chars(), DelimiterStyle::AsciiQuotedWords);
    if !word.closed {
        return None;
    }
    let end = text
        .char_indices()
        .nth(word.width)
        .map_or(text.len(), |(offset, _)| offset);
    let [delimiter] = split_quoted_words(&text[..end], Whitespace::Ascii)?
        .try_into()
        .ok()?;
    Some(Document {
        delimiter,
        strip_tabs,
    })
}

fn read_word(input: impl Iterator<Item = char>, style: DelimiterStyle) -> Word {
    let mut input = input.peekable();
    let quote = if style == DelimiterStyle::StripQuotes {
        None
    } else {
        input.next_if(|ch| matches!(ch, '\'' | '"'))
    };
    let mut word = Word {
        text: String::new(),
        width: usize::from(quote.is_some()),
        quoted: quote.is_some(),
        closed: quote.is_none(),
    };
    for ch in input {
        if quote == Some(ch) {
            word.width += 1;
            word.closed = true;
            break;
        }
        let is_whitespace = match style {
            DelimiterStyle::AsciiQuotedWords => ch.is_ascii_whitespace(),
            _ => ch.is_whitespace(),
        };
        if quote.is_none()
            && (is_whitespace
                || (style != DelimiterStyle::WhitespaceWord
                    && matches!(ch, ';' | '|' | '&' | '<' | '>' | '(' | ')')))
        {
            break;
        }
        word.width += 1;
        if style != DelimiterStyle::StripQuotes || !matches!(ch, '\\' | '\'' | '"') {
            word.text.push(ch);
        }
    }
    word
}
