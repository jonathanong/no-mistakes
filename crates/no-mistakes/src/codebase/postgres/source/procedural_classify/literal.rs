use super::classify;
use crate::codebase::postgres::source::types::{
    PostgresSqlPosition, PostgresSqlProceduralOccurrence, PostgresSqlProceduralOccurrenceKind,
    PostgresSqlSpan,
};
use sqlparser::{
    dialect::PostgreSqlDialect,
    tokenizer::{Token, TokenWithSpan, Tokenizer},
};

pub(super) fn command_kind(
    tokens: &[TokenWithSpan],
    command: &[usize],
) -> PostgresSqlProceduralOccurrenceKind {
    match literal_sql(tokens, command) {
        Some(sql) => script_kind(&sql),
        None => PostgresSqlProceduralOccurrenceKind::DynamicExecute,
    }
}

fn script_kind(sql: &str) -> PostgresSqlProceduralOccurrenceKind {
    let Ok(tokens) = Tokenizer::new(&PostgreSqlDialect {}, sql).tokenize_with_location() else {
        return PostgresSqlProceduralOccurrenceKind::Unknown;
    };
    let span = empty_span();
    let classified = classify(&tokens, &|_, _| span.clone(), false);
    fold(&classified.occurrences)
}

fn fold(occurrences: &[PostgresSqlProceduralOccurrence]) -> PostgresSqlProceduralOccurrenceKind {
    if occurrences.is_empty() {
        return PostgresSqlProceduralOccurrenceKind::Unknown;
    }
    let mut kind = PostgresSqlProceduralOccurrenceKind::Utility;
    for occurrence in occurrences {
        let nested = if occurrence.occurrences.is_empty() {
            occurrence.kind
        } else {
            prefer(occurrence.kind, fold(&occurrence.occurrences))
        };
        kind = prefer(kind, nested);
    }
    kind
}

fn prefer(
    left: PostgresSqlProceduralOccurrenceKind,
    right: PostgresSqlProceduralOccurrenceKind,
) -> PostgresSqlProceduralOccurrenceKind {
    if rank(right) > rank(left) {
        right
    } else {
        left
    }
}

fn rank(kind: PostgresSqlProceduralOccurrenceKind) -> u8 {
    match kind {
        PostgresSqlProceduralOccurrenceKind::Utility => 1,
        PostgresSqlProceduralOccurrenceKind::ControlFlow => 2,
        PostgresSqlProceduralOccurrenceKind::Dml => 3,
        PostgresSqlProceduralOccurrenceKind::Unknown => 4,
        PostgresSqlProceduralOccurrenceKind::DynamicExecute => 5,
    }
}

fn literal_sql(tokens: &[TokenWithSpan], command: &[usize]) -> Option<String> {
    let mut cursor = 0;
    let text = parse_literal(tokens, command, &mut cursor)?;
    (cursor == command.len()).then_some(text)
}

fn parse_literal(
    tokens: &[TokenWithSpan],
    command: &[usize],
    cursor: &mut usize,
) -> Option<String> {
    let mut text = parse_atom(tokens, command, cursor)?;
    while matches!(
        command.get(*cursor).map(|index| &tokens[*index].token),
        Some(Token::StringConcat)
    ) {
        *cursor += 1;
        text.push_str(&parse_atom(tokens, command, cursor)?);
    }
    Some(text)
}

fn parse_atom(tokens: &[TokenWithSpan], command: &[usize], cursor: &mut usize) -> Option<String> {
    let token = &tokens.get(*command.get(*cursor)?)?.token;
    if matches!(token, Token::LParen) {
        *cursor += 1;
        let text = parse_literal(tokens, command, cursor)?;
        let closing = &tokens.get(*command.get(*cursor)?)?.token;
        if !matches!(closing, Token::RParen) {
            return None;
        }
        *cursor += 1;
        return Some(text);
    }
    let value = string_value(token)?;
    *cursor += 1;
    Some(value)
}

fn string_value(token: &Token) -> Option<String> {
    match token {
        Token::SingleQuotedString(value)
        | Token::EscapedStringLiteral(value)
        | Token::NationalStringLiteral(value)
        | Token::UnicodeStringLiteral(value) => Some(value.clone()),
        Token::DollarQuotedString(value) => Some(value.value.clone()),
        _ => None,
    }
}

fn empty_span() -> PostgresSqlSpan {
    let position = PostgresSqlPosition {
        offset: 0,
        line: 1,
        column: 1,
    };
    PostgresSqlSpan {
        start: position.clone(),
        end: position,
    }
}
