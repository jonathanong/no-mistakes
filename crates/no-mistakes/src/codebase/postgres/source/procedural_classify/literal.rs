use super::classify;
use super::scan::{eq, word_of};
use crate::codebase::postgres::source::types::{
    PostgresSqlPosition, PostgresSqlProceduralOccurrence, PostgresSqlProceduralOccurrenceKind,
    PostgresSqlSpan,
};
use sqlparser::{
    dialect::PostgreSqlDialect,
    parser::Parser,
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
    // Offsets are token indexes for this inner walk. They are not source spans.
    let classified = classify(&tokens, &index_span, false);
    // Nested procedural children stay on the fail-closed path. Only this call
    // may promote a top-level SELECT, and only when that statement parses.
    fold(&tokens, &classified.occurrences, true)
}

fn fold(
    tokens: &[TokenWithSpan],
    occurrences: &[PostgresSqlProceduralOccurrence],
    promote_select: bool,
) -> PostgresSqlProceduralOccurrenceKind {
    if occurrences.is_empty() {
        return PostgresSqlProceduralOccurrenceKind::Unknown;
    }
    let mut kind = PostgresSqlProceduralOccurrenceKind::Utility;
    for occurrence in occurrences {
        let own = if promote_select {
            select_utility(tokens, occurrence)
        } else {
            occurrence.kind
        };
        let nested = if occurrence.occurrences.is_empty() {
            own
        } else {
            prefer(own, fold(tokens, &occurrence.occurrences, false))
        };
        kind = prefer(kind, nested);
    }
    kind
}

// Literal SELECT is utility here only. Bare SELECT in the outer walker stays unknown.
// A LOOP body is walker-only, so an unparsed SELECT would be reported complete.
fn select_utility(
    tokens: &[TokenWithSpan],
    occurrence: &PostgresSqlProceduralOccurrence,
) -> PostgresSqlProceduralOccurrenceKind {
    if occurrence.kind == PostgresSqlProceduralOccurrenceKind::Unknown
        && occurrence.occurrences.is_empty()
        && starts_with_select(tokens, occurrence)
        && select_parses(tokens, occurrence)
    {
        PostgresSqlProceduralOccurrenceKind::Utility
    } else {
        occurrence.kind
    }
}

fn select_parses(tokens: &[TokenWithSpan], occurrence: &PostgresSqlProceduralOccurrence) -> bool {
    let start = occurrence.span.start.offset;
    let end = occurrence.span.end.offset;
    if start > end || end >= tokens.len() {
        return false;
    }
    let mut statement = tokens[start..=end]
        .iter()
        .filter(|token| !matches!(token.token, Token::Whitespace(_)))
        .cloned()
        .collect::<Vec<_>>();
    // The span includes the statement terminator. A semicolon inside the
    // statement is still syntax, so only the final one is dropped.
    if matches!(
        statement.last().map(|token| &token.token),
        Some(Token::SemiColon)
    ) {
        statement.pop();
    }
    let mut parser = Parser::new(&PostgreSqlDialect {}).with_tokens_with_locations(statement);
    parser.parse_statement().is_ok() && parser.peek_token().token == Token::EOF
}

fn starts_with_select(
    tokens: &[TokenWithSpan],
    occurrence: &PostgresSqlProceduralOccurrence,
) -> bool {
    tokens
        .get(occurrence.span.start.offset)
        .and_then(|token| word_of(&token.token))
        .is_some_and(|word| eq(word, "SELECT"))
}

fn index_span(start: usize, end: usize) -> PostgresSqlSpan {
    PostgresSqlSpan {
        start: index_position(start),
        end: index_position(end),
    }
}

fn index_position(index: usize) -> PostgresSqlPosition {
    PostgresSqlPosition {
        offset: index,
        line: 1,
        column: 1,
    }
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
