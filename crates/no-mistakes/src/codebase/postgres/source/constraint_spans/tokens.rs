use super::super::{locations::Locations, types::PostgresSqlSpan};
use sqlparser::tokenizer::{Token, TokenWithSpan};

pub(super) type Range = (usize, usize);

pub(super) fn significant(
    tokens: &[TokenWithSpan],
) -> impl Iterator<Item = (usize, &TokenWithSpan)> {
    tokens
        .iter()
        .enumerate()
        .filter(|(_, token)| !matches!(token.token, Token::Whitespace(_)))
}

pub(super) fn word(token: &TokenWithSpan, value: &str) -> bool {
    matches!(&token.token, Token::Word(word) if word.quote_style.is_none() && word.value.eq_ignore_ascii_case(value))
}

pub(super) fn top_level(tokens: &[TokenWithSpan], range: Range) -> Vec<usize> {
    let mut depth = 0usize;
    let mut brackets = 0usize;
    let mut result = Vec::new();
    for (index, token) in tokens.iter().enumerate().take(range.1).skip(range.0) {
        match token.token {
            Token::RParen => depth = depth.saturating_sub(1),
            Token::RBracket => brackets = brackets.saturating_sub(1),
            _ => {}
        }
        if depth == 0 && brackets == 0 {
            result.push(index);
        }
        match token.token {
            Token::LParen => depth += 1,
            Token::LBracket => brackets += 1,
            _ => {}
        }
    }
    result
}

pub(super) fn next_significant(
    tokens: &[TokenWithSpan],
    start: usize,
    end: usize,
) -> Option<usize> {
    (start..end).find(|position| !matches!(tokens[*position].token, Token::Whitespace(_)))
}

pub(super) fn is_referential_action(
    tokens: &[TokenWithSpan],
    position: usize,
    start: usize,
) -> bool {
    matches!(tokens.get(position).map(|token| &token.token), Some(Token::Word(word)) if word.value.eq_ignore_ascii_case("DEFAULT") || word.value.eq_ignore_ascii_case("NULL"))
        && position > start
        && (start..position)
            .rev()
            .find(|prior| !matches!(tokens[*prior].token, Token::Whitespace(_)))
            .is_some_and(|prior| word(&tokens[prior], "SET"))
}

pub(super) fn paren(token: &TokenWithSpan, open: bool) -> bool {
    token.token == if open { Token::LParen } else { Token::RParen }
}

pub(super) fn definitions(tokens: &[TokenWithSpan], depth_start: usize) -> Vec<Range> {
    let mut ranges = Vec::new();
    let mut depth = 0usize;
    let mut brackets = 0usize;
    let mut start = None;
    for (index, token) in significant(tokens) {
        if token.token == Token::LBracket {
            brackets += 1;
        }
        if token.token == Token::RBracket {
            brackets = brackets.saturating_sub(1);
        }
        if paren(token, true) {
            depth += 1;
            if depth == depth_start {
                start = Some(index + 1);
            }
        } else if paren(token, false) {
            if depth == depth_start {
                let begin = start
                    .take()
                    .expect("closed definition list starts at an opening paren");
                if begin < index {
                    ranges.push((begin, index));
                }
            }
            depth = depth.saturating_sub(1);
        } else if token.token == Token::Comma && depth == depth_start && brackets == 0 {
            let begin = start
                .replace(index + 1)
                .expect("definition commas follow an opening paren");
            if begin < index {
                ranges.push((begin, index));
            }
        }
    }
    ranges
}

pub(in crate::codebase::postgres::source) fn table_ranges(tokens: &[TokenWithSpan]) -> Vec<Range> {
    // CREATE TABLE's first definition list starts at the first parenthesis.
    definitions(tokens, 1)
}

pub(super) fn enclosing(
    ranges: &[Range],
    tokens: &[TokenWithSpan],
    span: sqlparser::tokenizer::Span,
) -> Option<Range> {
    ranges.iter().copied().find(|(start, end)| {
        tokens.get(*start..*end).is_some_and(|part| {
            part.iter()
                .any(|token| token.span.start <= span.start && span.start <= token.span.end)
        })
    })
}

pub(super) fn output(
    range: Range,
    tokens: &[TokenWithSpan],
    locations: &Locations<'_>,
) -> Option<PostgresSqlSpan> {
    let part = tokens.get(range.0..range.1)?;
    let mut significant = part
        .iter()
        .filter(|token| !matches!(token.token, Token::Whitespace(_)));
    let first = significant.next()?;
    let last = significant.next_back().unwrap_or(first);
    locations.span(sqlparser::tokenizer::Span {
        start: first.span.start,
        end: last.span.end,
    })
}

#[cfg(test)]
mod tests;
