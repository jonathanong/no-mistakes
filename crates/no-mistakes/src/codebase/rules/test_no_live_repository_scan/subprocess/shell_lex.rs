use std::iter::Peekable;
use std::str::Chars;

pub(super) struct Word {
    pub(super) text: String,
    pub(super) assignment: bool,
}

pub(super) enum Sep {
    And,
    Semi,
    Or,
}

pub(super) struct Part {
    pub(super) words: Vec<Word>,
    pub(super) next: Option<Sep>,
}

/// Simple commands split on unquoted `&&`, `;`, and `||`. `$`, a backtick, and
/// `\` are static only inside single quotes.
pub(super) fn segments(text: &str) -> Option<Vec<Part>> {
    let mut chars = text.chars().peekable();
    let mut parts = Vec::new();
    let mut words = Vec::new();
    loop {
        skip_ws(&mut chars);
        if chars.peek().is_none() {
            parts.push(Part { words, next: None });
            break;
        }
        if let Some(sep) = take_sep(&mut chars) {
            parts.push(Part {
                words: std::mem::take(&mut words),
                next: Some(sep),
            });
            continue;
        }
        words.push(read_word(&mut chars)?);
    }
    Some(parts)
}

fn skip_ws(chars: &mut Peekable<Chars<'_>>) {
    while chars
        .peek()
        .is_some_and(|character| character.is_whitespace())
    {
        chars.next();
    }
}

fn take_sep(chars: &mut Peekable<Chars<'_>>) -> Option<Sep> {
    let sep = sep_at(chars)?;
    chars.next();
    if !matches!(sep, Sep::Semi) {
        chars.next();
    }
    Some(sep)
}

fn sep_at(chars: &Peekable<Chars<'_>>) -> Option<Sep> {
    let mut look = chars.clone();
    match (look.next(), look.next()) {
        (Some(';'), _) => Some(Sep::Semi),
        (Some('&'), Some('&')) => Some(Sep::And),
        (Some('|'), Some('|')) => Some(Sep::Or),
        _ => None,
    }
}

fn read_word(chars: &mut Peekable<Chars<'_>>) -> Option<Word> {
    let mut word = Word {
        text: String::new(),
        assignment: false,
    };
    let mut name_open = true;
    while let Some(character) = chars.peek().copied() {
        if character.is_whitespace() || sep_at(chars).is_some() {
            break;
        }
        if character == '\\' {
            return None;
        }
        if character == '\'' || character == '"' {
            name_open = false;
            read_quoted(chars, &mut word)?;
            continue;
        }
        if matches!(character, '`' | '$') {
            return None;
        }
        chars.next();
        if name_open && character == '=' {
            word.assignment = identifier(&word.text);
            name_open = false;
        }
        word.text.push(character);
    }
    Some(word)
}

fn read_quoted(chars: &mut Peekable<Chars<'_>>, word: &mut Word) -> Option<()> {
    let quote = chars.next()?;
    loop {
        let inner = chars.next()?;
        if inner == quote {
            return Some(());
        }
        if inner == '\\' || inner == '\n' || inner == '\r' {
            return None;
        }
        if quote == '"' && matches!(inner, '`' | '$') {
            return None;
        }
        word.text.push(inner);
    }
}

fn identifier(text: &str) -> bool {
    let mut chars = text.chars();
    match chars.next() {
        Some(first) if first.is_ascii_alphabetic() || first == '_' => {
            chars.all(|character| character.is_ascii_alphanumeric() || character == '_')
        }
        _ => false,
    }
}
