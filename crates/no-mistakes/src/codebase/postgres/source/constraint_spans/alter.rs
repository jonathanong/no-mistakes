use super::super::{locations::Locations, types::PostgresSqlSpan};
use super::tokens::{output, paren, significant, word};
use sqlparser::{
    ast::{AlterTableOperation, Spanned},
    tokenizer::{Token, TokenWithSpan},
};

pub(in crate::codebase::postgres::source) fn alter_ranges(
    tokens: &[TokenWithSpan],
    table_name_end: sqlparser::tokenizer::Location,
) -> Vec<(usize, usize)> {
    // AST operation order pairs every ALTER variant with its outer comma range;
    // do not infer the operation vocabulary from a hand-maintained keyword set.
    let mut depth = 0usize;
    let mut brackets = 0usize;
    let mut ranges = Vec::new();
    let Some((start, _)) =
        significant(tokens).find(|(_, token)| token.span.start >= table_name_end)
    else {
        return ranges;
    };
    let mut operation_start = start;
    for (position, token) in significant(tokens).filter(|(position, _)| *position >= start) {
        if paren(token, true) {
            depth += 1;
        }
        if token.token == Token::LBracket {
            brackets += 1;
        }
        if token.token == Token::RBracket {
            brackets = brackets.saturating_sub(1);
        }
        if token.token == Token::Comma && depth == 0 && brackets == 0 {
            ranges.push((operation_start, position));
            operation_start = position + 1;
        }
        if paren(token, false) {
            depth = depth.saturating_sub(1);
        }
    }
    let end = tokens
        .iter()
        .rposition(|token| {
            !matches!(token.token, Token::Whitespace(_)) && token.token != Token::SemiColon
        })
        .expect("parsed ALTER TABLE has an operation token")
        + 1;
    ranges.push((operation_start, end));
    ranges
}

pub(in crate::codebase::postgres::source) fn alter_span(
    index: usize,
    operation: &AlterTableOperation,
    tokens: &[TokenWithSpan],
    ranges: &[(usize, usize)],
    locations: &Locations<'_>,
) -> Option<PostgresSqlSpan> {
    if !matches!(operation, AlterTableOperation::AddConstraint { .. }) {
        return None;
    }
    let range = *ranges.get(index)?;
    let ast_span = operation.span();
    if !tokens
        .get(range.0..range.1)?
        .iter()
        .any(|token| token.span.start <= ast_span.start && ast_span.start <= token.span.end)
    {
        return None;
    }
    let start = (range.0..range.1).find(|position| {
        word(&tokens[*position], "CONSTRAINT")
            || word(&tokens[*position], "PRIMARY")
            || word(&tokens[*position], "UNIQUE")
            || word(&tokens[*position], "FOREIGN")
            || word(&tokens[*position], "CHECK")
    })?;
    output((start, range.1), tokens, locations)
}
