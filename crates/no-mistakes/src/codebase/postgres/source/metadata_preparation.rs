//! Metadata comments borrow the request parser before its conditional AST pass.
use super::{locations::Locations, types::PostgresSqlStatementKind};
use sqlparser::{
    keywords::Keyword,
    parser::Parser,
    tokenizer::{Location, Token, Whitespace},
};
use std::collections::BTreeMap;

pub(super) struct Comment {
    pub facts: Option<Result<PostgresSqlStatementKind, String>>,
    pub end: Location,
}
pub(super) type Comments = BTreeMap<Location, Comment>;

pub(super) fn prepare<'a>(
    mut parser: Parser<'a>,
    locations: &Locations<'_>,
) -> (Parser<'a>, Comments) {
    let mut tokens = Vec::new();
    while parser.token_at(tokens.len()).token != Token::EOF {
        tokens.push(parser.token_at(tokens.len()).clone());
    }
    let significant = tokens
        .iter()
        .enumerate()
        .filter_map(|(index, token)| {
            (!matches!(token.token, Token::Whitespace(_))).then_some(index)
        })
        .collect::<Vec<_>>();
    let mut comments = Comments::new();
    for (position, prefix) in significant.windows(3).enumerate() {
        let keyword = |index: usize, expected: Keyword| matches!(&tokens[index].token, Token::Word(word) if word.quote_style.is_none() && word.keyword == expected);
        if !keyword(prefix[0], Keyword::COMMENT)
            || !keyword(prefix[1], Keyword::ON)
            || !statement_position(&tokens, &significant, position)
        {
            continue;
        }
        let start = prefix[0];
        let end = tokens[start..]
            .iter()
            .position(|token| token.token == Token::SemiColon
                || matches!(&token.token, Token::Word(word) if word.quote_style.is_none() && word.keyword == Keyword::END))
            .map_or(tokens.len(), |offset| start + offset);
        while parser.index() > start {
            parser.prev_token();
        }
        while parser.index() < start {
            parser.next_token_no_skip();
        }
        let mut facts = super::metadata::collect(&mut parser, locations);
        if parser.index() > end
            || !matches!(parser.peek_token().token, Token::SemiColon | Token::EOF)
        {
            facts = Err("COMMENT has no complete original statement boundary".into());
        }
        let original_end = tokens[start..end]
            .iter()
            .filter(|token| !matches!(token.token, Token::Whitespace(_)))
            .map(|token| token.span.end)
            .max()
            .unwrap_or(tokens[start].span.end);
        comments.insert(
            tokens[start].span.start,
            Comment {
                facts: Some(facts),
                end: original_end,
            },
        );
        // Only the native conditional grammar sees SELECT 0. The real metadata
        // was collected once above; every original position remains unchanged.
        tokens[start].token = Token::make_word("SELECT", None);
        tokens[prefix[1]].token = Token::Number("0".into(), false);
        for token in &mut tokens[prefix[1] + 1..end] {
            token.token = Token::Whitespace(Whitespace::Space);
        }
    }
    (parser.with_tokens_with_locations(tokens), comments)
}

/// COMMENT is also an ordinary table/alias name. Only statement and wrapper
/// child positions may borrow the metadata grammar and mask original tokens.
fn statement_position(
    tokens: &[sqlparser::tokenizer::TokenWithSpan],
    significant: &[usize],
    position: usize,
) -> bool {
    if position == 0 || boundary(tokens, significant, position - 1) {
        return true;
    }
    let first = (0..position)
        .rev()
        .find(|previous| boundary(tokens, significant, *previous))
        .map_or(0, |previous| previous + 1);
    prepared_child_position(tokens, significant[first], significant[position])
        == Some(significant[position])
}

fn prepared_child_position(
    tokens: &[sqlparser::tokenizer::TokenWithSpan],
    mut start: usize,
    limit: usize,
) -> Option<usize> {
    for _ in 0..16 {
        if start >= limit {
            return Some(start);
        }
        let kind = match &tokens[start].token {
            Token::Word(word) if word.quote_style.is_none() && word.keyword == Keyword::EXPLAIN => {
                super::types::PostgresSqlWrapperKind::Explain
            }
            Token::Word(word) if word.quote_style.is_none() && word.keyword == Keyword::PREPARE => {
                super::types::PostgresSqlWrapperKind::Prepare
            }
            _ => return Some(start),
        };
        start += super::wrappers::prepared_child_start(&tokens[start..], kind);
        while tokens
            .get(start)
            .is_some_and(|token| matches!(token.token, Token::Whitespace(_)))
        {
            start += 1;
        }
    }
    None
}

fn boundary(
    tokens: &[sqlparser::tokenizer::TokenWithSpan],
    significant: &[usize],
    position: usize,
) -> bool {
    let keyword = |at: usize, expected: Keyword| {
        significant.get(at).is_some_and(|index| matches!(&tokens[*index].token, Token::Word(word) if word.quote_style.is_none() && word.keyword == expected))
    };
    if tokens[significant[position]].token == Token::SemiColon
        || keyword(position, Keyword::THEN)
        || keyword(position, Keyword::ELSE)
    {
        return true;
    }
    if keyword(position, Keyword::BEGIN) {
        return position == 0
            || tokens[significant[position - 1]].token == Token::SemiColon
            || keyword(position - 1, Keyword::THEN)
            || keyword(position - 1, Keyword::ELSE);
    }
    if keyword(position, Keyword::ATOMIC) && position > 0 && keyword(position - 1, Keyword::BEGIN) {
        if boundary(tokens, significant, position - 1) {
            return true;
        }
        let first = significant[..position - 1]
            .iter()
            .rposition(|index| tokens[*index].token == Token::SemiColon)
            .map_or(0, |previous| previous + 1);
        // CREATE FUNCTION headers own BEGIN ATOMIC; an unreserved table named
        // BEGIN or ATOMIC in SELECT must never establish a statement boundary.
        return prepared_child_position(tokens, significant[first], significant[position - 1])
            .is_some_and(|child| {
                let first = significant.partition_point(|index| *index < child);
                keyword(first, Keyword::CREATE)
                    && (keyword(first + 1, Keyword::FUNCTION)
                        || (keyword(first + 1, Keyword::OR)
                            && keyword(first + 2, Keyword::REPLACE)
                            && keyword(first + 3, Keyword::FUNCTION)))
            });
    }
    false
}
