use sqlparser::dialect::{Dialect, PostgreSqlDialect};
use sqlparser::tokenizer::{Token, TokenWithSpan, Tokenizer};

#[derive(Debug)]
struct RawPostgresDialect;

impl Dialect for RawPostgresDialect {
    fn is_delimited_identifier_start(&self, character: char) -> bool {
        character == '"'
    }
    fn is_identifier_start(&self, character: char) -> bool {
        character.is_alphabetic() || character == '_' || !character.is_ascii()
    }
    fn is_identifier_part(&self, character: char) -> bool {
        character.is_alphanumeric() || matches!(character, '$' | '_') || !character.is_ascii()
    }
    fn supports_nested_comments(&self) -> bool {
        true
    }
    fn supports_string_escape_constant(&self) -> bool {
        true
    }
}

struct RawUnicodeLiteral {
    start_line: u64,
    start_column: u64,
    content_start: usize,
    content_end: usize,
    content: String,
    escape: char,
}

pub(super) fn tokenize(sql: &str) -> Vec<Token> {
    tokenize_with_location(sql, false)
        .into_iter()
        .map(|token| token.token)
        .collect()
}

pub(crate) fn tokenize_raw_unicode(sql: &str) -> Vec<TokenWithSpan> {
    tokenize_with_location(sql, true)
}

fn tokenize_with_location(sql: &str, raw_unicode: bool) -> Vec<TokenWithSpan> {
    let Some((masked, literals)) = mask_literals(sql) else {
        return Vec::new();
    };
    let Ok(mut tokens) = Tokenizer::new(&PostgreSqlDialect {}, &masked).tokenize_with_location()
    else {
        return Vec::new();
    };
    for token in &mut tokens {
        let Some(literal) = literals.iter().find(|literal| {
            token.span.start.line == literal.start_line
                && token.span.start.column == literal.start_column
        }) else {
            continue;
        };
        if matches!(token.token, Token::UnicodeStringLiteral(_)) {
            let content = if raw_unicode {
                Some(literal.content.clone())
            } else {
                decode_unicode_string(&literal.content, literal.escape)
            };
            let Some(content) = content else {
                return Vec::new();
            };
            token.token = Token::UnicodeStringLiteral(content);
        }
    }
    tokens
}

fn mask_literals(sql: &str) -> Option<(String, Vec<RawUnicodeLiteral>)> {
    let tokens = Tokenizer::new(&RawPostgresDialect, sql)
        .with_unescape(false)
        .tokenize_with_location()
        .ok()?;
    let mut literals = Vec::new();
    for (index, window) in tokens.windows(3).enumerate() {
        let [prefix, ampersand, literal] = window else {
            continue;
        };
        if !word(prefix, "U")
            || !matches!(ampersand.token, Token::Ampersand)
            || prefix.span.end != ampersand.span.start
            || ampersand.span.end != literal.span.start
        {
            continue;
        }
        let Token::SingleQuotedString(content) = &literal.token else {
            continue;
        };
        let content_start =
            location_offset(sql, literal.span.start.line, literal.span.start.column)? + 1;
        let content_end = location_offset(sql, literal.span.end.line, literal.span.end.column)? - 1;
        literals.push(RawUnicodeLiteral {
            start_line: prefix.span.start.line,
            start_column: prefix.span.start.column,
            content_start,
            content_end,
            content: content.clone(),
            escape: unicode_escape_after(&tokens, index + 3).unwrap_or('\\'),
        });
    }
    let mut masked = sql.to_owned();
    for literal in literals.iter().rev() {
        let replacement = sql
            .get(literal.content_start..literal.content_end)?
            .chars()
            .map(|character| match character {
                '\n' | '\r' | '\'' => character,
                _ => 'a',
            })
            .collect::<String>();
        masked.replace_range(literal.content_start..literal.content_end, &replacement);
    }
    Some((masked, literals))
}

fn unicode_escape_after(tokens: &[TokenWithSpan], start: usize) -> Option<char> {
    let mut tokens = tokens[start..]
        .iter()
        .filter(|token| !matches!(token.token, Token::Whitespace(_)));
    if !word(tokens.next()?, "UESCAPE") {
        return None;
    }
    let Token::SingleQuotedString(value) = &tokens.next()?.token else {
        return None;
    };
    let mut chars = value.chars();
    let escape = chars.next()?;
    chars.next().is_none().then_some(escape)
}

fn word(token: &TokenWithSpan, expected: &str) -> bool {
    matches!(&token.token, Token::Word(word) if word.value.eq_ignore_ascii_case(expected))
}

fn location_offset(sql: &str, line: u64, column: u64) -> Option<usize> {
    let line_start = sql
        .split_inclusive('\n')
        .take(line.saturating_sub(1) as usize)
        .map(str::len)
        .sum::<usize>();
    let current_line = sql.get(line_start..)?.split('\n').next()?;
    let column_offset = current_line
        .char_indices()
        .nth(column.saturating_sub(1) as usize)
        .map_or(current_line.len(), |(offset, _)| offset);
    Some(line_start + column_offset)
}

pub(crate) fn decode_unicode_string(value: &str, escape: char) -> Option<String> {
    super::unicode_decode::decode(value, escape)
}
