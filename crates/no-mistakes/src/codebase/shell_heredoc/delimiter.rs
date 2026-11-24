use super::{first_document, Document};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum DelimiterStyle {
    StripQuotes,
    LeadingQuote,
    WhitespaceWord,
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
        if quote.is_none()
            && (ch.is_whitespace()
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
