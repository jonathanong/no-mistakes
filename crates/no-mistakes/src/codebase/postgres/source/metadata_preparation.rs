//! Metadata comments borrow the request parser before its conditional AST pass.
use super::{locations::Locations, types::PostgresSqlStatementKind};
use sqlparser::{
    keywords::Keyword,
    parser::Parser,
    tokenizer::{Location, Token, Whitespace},
};
use std::collections::HashMap;

pub(super) struct Comment {
    pub facts: Option<Result<PostgresSqlStatementKind, String>>,
    pub end: Location,
}
pub(super) type Comments = HashMap<Location, Comment>;

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
    for prefix in significant.windows(3) {
        let keyword = |index: usize, expected: Keyword| matches!(&tokens[index].token, Token::Word(word) if word.quote_style.is_none() && word.keyword == expected);
        if !keyword(prefix[0], Keyword::COMMENT) || !keyword(prefix[1], Keyword::ON) {
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
