//! Delimit only conflict clauses paired with a top-level DO action.
use super::keyword;
use sqlparser::{
    keywords::Keyword,
    tokenizer::{Location, Token, TokenWithSpan},
};

pub(in crate::codebase::postgres::source) fn prepare(
    tokens: &mut [TokenWithSpan],
) -> Vec<Location> {
    let significant = tokens
        .iter()
        .enumerate()
        .filter_map(|(index, token)| {
            (!matches!(token.token, Token::Whitespace(_))).then_some(index)
        })
        .collect::<Vec<_>>();
    let eligible = eligible(tokens, &significant);
    let mut action = false;
    let mut markers = Vec::new();
    for position in (0..significant.len()).rev() {
        let index = significant[position];
        if tokens[index].token == Token::SemiColon {
            action = false;
        }
        if !eligible[position] {
            continue;
        }
        let next = significant
            .get(position + 1)
            .map(|index| &tokens[*index].token);
        if keyword(&tokens[index].token, Keyword::DO)
            && next.is_some_and(|token| {
                keyword(token, Keyword::NOTHING) || keyword(token, Keyword::UPDATE)
            })
        {
            action = true;
        }
        if action
            && keyword(&tokens[index].token, Keyword::ON)
            && next.is_some_and(|token| keyword(token, Keyword::CONFLICT))
        {
            // The nearest ON CONFLICT before DO belongs to the upsert. Earlier
            // JOIN ... ON conflict conditions retain their original tokens.
            markers.push(tokens[index].span.start);
            tokens[index].token = Token::SemiColon;
            action = false;
        }
    }
    markers.reverse();
    markers
}

fn eligible(tokens: &[TokenWithSpan], significant: &[usize]) -> Vec<bool> {
    let mut beginning = true;
    let mut with = false;
    let mut insert = false;
    let mut depth: usize = 0;
    significant
        .iter()
        .enumerate()
        .map(|(position, index)| {
            let token = &tokens[*index].token;
            if *token == Token::SemiColon {
                depth = 0;
                beginning = true;
                insert = false;
                return false;
            }
            if depth == 0
                && !insert
                && keyword(token, Keyword::ATOMIC)
                && position > 0
                && keyword(&tokens[significant[position - 1]].token, Keyword::BEGIN)
                && super::super::super::metadata_preparation::boundary(
                    tokens,
                    significant,
                    position,
                )
            {
                beginning = true;
                insert = false;
                return false;
            }
            if beginning {
                with = keyword(token, Keyword::WITH)
                    || keyword(token, Keyword::EXPLAIN)
                    || keyword(token, Keyword::PREPARE);
                insert = keyword(token, Keyword::INSERT);
                beginning = false;
            } else if with && depth == 0 && keyword(token, Keyword::INSERT) {
                insert = true;
            }
            let eligible = insert && depth == 0;
            match token {
                Token::LParen => depth += 1,
                Token::RParen => depth = depth.saturating_sub(1),
                _ => {}
            }
            eligible
        })
        .collect()
}
