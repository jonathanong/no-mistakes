use super::model::{PartitionKey, PartitionKeyElement, PartitionStrategy};

#[derive(Debug)]
pub(super) enum PartitionParseError {
    Strategy(String),
    Syntax,
}

pub(super) fn parse_partition_key(raw: &str) -> Result<PartitionKey, PartitionParseError> {
    let raw = raw.trim();
    let split_at = raw
        .find(|character: char| character.is_whitespace() || character == '(')
        .unwrap_or(raw.len());
    let strategy_word = &raw[..split_at];
    if strategy_word.is_empty() {
        return Err(PartitionParseError::Syntax);
    }
    let strategy = match strategy_word.to_ascii_lowercase().as_str() {
        "range" => PartitionStrategy::Range,
        "list" => PartitionStrategy::List,
        "hash" => PartitionStrategy::Hash,
        _ => return Err(PartitionParseError::Strategy(strategy_word.to_string())),
    };
    let inner = paren_inner(raw[split_at..].trim())?;
    let elements = split_elements(inner)?;
    Ok(PartitionKey { strategy, elements })
}

fn paren_inner(raw: &str) -> Result<&str, PartitionParseError> {
    let mut scan = Scan::new(raw);
    if scan.bump() != Some('(') {
        return Err(PartitionParseError::Syntax);
    }
    let start = scan.index;
    let mut depth = 1i32;
    while let Some(character) = scan.bump() {
        if scan.quote.is_some() {
            continue;
        }
        match character {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    let end = scan.index - ')'.len_utf8();
                    if scan.bump().is_some() {
                        return Err(PartitionParseError::Syntax);
                    }
                    return Ok(&raw[start..end]);
                }
            }
            _ => {}
        }
    }
    Err(PartitionParseError::Syntax)
}

fn split_elements(raw: &str) -> Result<Vec<PartitionKeyElement>, PartitionParseError> {
    let mut scan = Scan::new(raw);
    let mut start = 0;
    let mut depth = 0i32;
    let mut parts = Vec::new();
    while let Some(character) = scan.bump() {
        if scan.quote.is_some() {
            continue;
        }
        match character {
            '(' => depth += 1,
            ')' => depth -= 1,
            ',' if depth == 0 => {
                parts.push(&raw[start..scan.index - 1]);
                start = scan.index;
            }
            _ => {}
        }
    }
    parts.push(&raw[start..]);
    if parts.iter().any(|part| part.trim().is_empty()) {
        return Err(PartitionParseError::Syntax);
    }
    Ok(parts.into_iter().map(classify).collect())
}

fn classify(raw: &str) -> PartitionKeyElement {
    let trimmed = raw.trim();
    if let Some(name) = column_name(trimmed) {
        PartitionKeyElement::Column(name)
    } else {
        PartitionKeyElement::Expression(trimmed.to_string())
    }
}

/// A key element that is a column, with or without the `COLLATE <collation>` and operator class
/// that `pg_get_partkeydef` writes after it: `name COLLATE "C" text_pattern_ops`.
fn column_name(raw: &str) -> Option<String> {
    let (name, rest) = ident(raw)?;
    modifiers_only(rest).then_some(name)
}

fn modifiers_only(rest: &str) -> bool {
    if !rest.is_empty() && !rest.starts_with(char::is_whitespace) {
        return false;
    }
    let mut rest = rest.trim_start();
    let word_end = rest.find(char::is_whitespace).unwrap_or(rest.len());
    if rest[..word_end].eq_ignore_ascii_case("collate") {
        match skip_qualified(rest[word_end..].trim_start()) {
            Some(after) => rest = after.trim_start(),
            None => return false,
        }
    }
    rest.is_empty() || skip_qualified(rest).is_some_and(|after| after.trim().is_empty())
}

/// `schema.name` or `name`, each part bare or quoted: what follows it.
fn skip_qualified(mut raw: &str) -> Option<&str> {
    loop {
        let (_, rest) = ident(raw)?;
        match rest.strip_prefix('.') {
            Some(next) => raw = next,
            None => return Some(rest),
        }
    }
}

/// The identifier at the start of `raw`, decoded the way PostgreSQL reads it (a bare name folds
/// to lower case), and the text after it.
fn ident(raw: &str) -> Option<(String, &str)> {
    if let Some(inner) = raw.strip_prefix('"') {
        return quoted_ident(inner);
    }
    let first = raw.chars().next()?;
    if !(first.is_ascii_alphabetic() || first == '_') {
        return None;
    }
    let end = raw
        .find(|character: char| {
            !(character.is_ascii_alphanumeric() || character == '_' || character == '$')
        })
        .unwrap_or(raw.len());
    Some((raw[..end].to_ascii_lowercase(), &raw[end..]))
}

fn quoted_ident(raw: &str) -> Option<(String, &str)> {
    let mut chars = raw.char_indices();
    let mut name = String::new();
    while let Some((index, character)) = chars.next() {
        if character == '"' {
            if raw[index + 1..].starts_with('"') {
                name.push('"');
                chars.next();
                continue;
            }
            return Some((name, &raw[index + 1..]));
        }
        name.push(character);
    }
    None
}

struct Scan<'a> {
    raw: &'a str,
    index: usize,
    quote: Option<char>,
}

impl<'a> Scan<'a> {
    fn new(raw: &'a str) -> Self {
        Self {
            raw,
            index: 0,
            quote: None,
        }
    }

    fn bump(&mut self) -> Option<char> {
        let character = self.raw[self.index..].chars().next()?;
        self.index += character.len_utf8();
        if let Some(quote) = self.quote {
            if character == quote {
                if self.raw[self.index..].starts_with(quote) {
                    self.index += quote.len_utf8();
                } else {
                    self.quote = None;
                }
            }
            return Some(character);
        }
        if character == '\'' || character == '"' {
            self.quote = Some(character);
        }
        Some(character)
    }
}
