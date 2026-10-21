//! A conflict-target `WHERE` inside parentheses cannot become a semicolon.
//! That delimiter would end the enclosing CTE. The predicate is parsed from a
//! copy, then the original tokens are blank so the shared grammar sees `DO`.
use super::expressions::{self, Located};
use super::ConflictFacts;
use crate::codebase::postgres::source::dialect::PostgresSourceDialect;
use sqlparser::{
    ast::{Insert, Spanned},
    keywords::Keyword,
    parser::Parser,
    tokenizer::{Location, Span, Token, TokenWithSpan, Whitespace},
};

pub(super) fn keyword(token: &Token, expected: Keyword) -> bool {
    matches!(token, Token::Word(word) if word.quote_style.is_none() && word.keyword == expected)
}

pub(in crate::codebase::postgres::source) struct CtePredicate {
    pub on_start: Location,
    pub predicate: Located,
}

impl CtePredicate {
    pub(in crate::codebase::postgres::source) fn owns(&self, insert: &Insert) -> bool {
        contains(insert.span(), self.on_start)
            && insert
                .source
                .as_ref()
                .is_none_or(|source| !contains(source.span(), self.on_start))
    }

    pub(in crate::codebase::postgres::source) fn facts(&self) -> ConflictFacts {
        ConflictFacts {
            expressions: Vec::new(),
            assignments: Vec::new(),
            predicate: Some(self.predicate.clone()),
            action_predicate: None,
            span: None,
            source_span: None,
            unsupported_with: false,
        }
    }
}

fn contains(span: Span, location: Location) -> bool {
    let before =
        |left: Location, right: Location| (left.line, left.column) < (right.line, right.column);
    !before(location, span.start) && before(location, span.end)
}

pub(super) fn extract(tokens: &mut [TokenWithSpan]) -> Vec<CtePredicate> {
    let significant = significant(tokens);
    let mut found = Vec::new();
    let mut depth = 0usize;
    let mut inserts = Vec::new();
    let mut index = 0usize;
    while index < significant.len() {
        let token_index = significant[index];
        let token = &tokens[token_index].token;
        if *token == Token::SemiColon {
            // An unmatched parenthesis does not survive a statement delimiter.
            depth = 0;
            inserts.clear();
        }
        if depth > 0 && keyword(token, Keyword::INSERT) {
            inserts.push(depth);
        }
        if inserts.last() == Some(&depth)
            && keyword(token, Keyword::ON)
            && next_is(tokens, &significant, index, Keyword::CONFLICT)
        {
            if let Some((start, end)) = target_where(tokens, &significant, index, depth) {
                let copied = predicate_tokens(tokens, &significant, start, end);
                if let Some(predicate) = parse_predicate(copied) {
                    blank(tokens, &significant, start, end);
                    found.push(CtePredicate {
                        on_start: tokens[token_index].span.start,
                        predicate,
                    });
                }
            }
        }
        match &tokens[token_index].token {
            Token::LParen => depth += 1,
            Token::RParen => {
                depth = depth.saturating_sub(1);
                while inserts.last().is_some_and(|insert| *insert > depth) {
                    inserts.pop();
                }
            }
            _ => {}
        }
        index += 1;
    }
    found
}

fn target_where(
    tokens: &[TokenWithSpan],
    significant: &[usize],
    on: usize,
    insert_depth: usize,
) -> Option<(usize, usize)> {
    // Only `( columns ) WHERE` bears a predicate; `ON CONSTRAINT` and a bare
    // `WHERE` are invalid and must keep failing closed.
    if tokens[*significant.get(on + 2)?].token != Token::LParen {
        return None;
    }
    let mut depth = insert_depth + 1;
    let mut position = on + 3;
    while depth > insert_depth {
        match &tokens[*significant.get(position)?].token {
            Token::LParen => depth += 1,
            Token::RParen => depth -= 1,
            _ => {}
        }
        position += 1;
    }
    if !keyword(&tokens[*significant.get(position)?].token, Keyword::WHERE) {
        return None;
    }
    let where_at = position;
    position += 1;
    while position < significant.len() {
        let token = &tokens[significant[position]].token;
        if depth == insert_depth && keyword(token, Keyword::DO) {
            return Some((where_at, position));
        }
        match token {
            Token::LParen => depth += 1,
            Token::RParen if depth == insert_depth => return None,
            Token::RParen => depth = depth.saturating_sub(1),
            _ => {}
        }
        position += 1;
    }
    None
}

fn predicate_tokens(
    tokens: &[TokenWithSpan],
    significant: &[usize],
    start: usize,
    end: usize,
) -> Vec<TokenWithSpan> {
    significant[start + 1..end]
        .iter()
        .map(|index| tokens[*index].clone())
        .collect()
}

fn parse_predicate(tokens: Vec<TokenWithSpan>) -> Option<Located> {
    let mut parser = Parser::new(&PostgresSourceDialect).with_tokens_with_locations(tokens);
    let predicate = expressions::parse(&mut parser).ok()?;
    matches!(parser.peek_token().token, Token::EOF).then_some(predicate)
}

fn blank(tokens: &mut [TokenWithSpan], significant: &[usize], start: usize, end: usize) {
    for index in &significant[start..end] {
        tokens[*index].token = Token::Whitespace(Whitespace::Space);
    }
}

fn next_is(
    tokens: &[TokenWithSpan],
    significant: &[usize],
    index: usize,
    expected: Keyword,
) -> bool {
    significant
        .get(index + 1)
        .is_some_and(|token| keyword(&tokens[*token].token, expected))
}

fn significant(tokens: &[TokenWithSpan]) -> Vec<usize> {
    tokens
        .iter()
        .enumerate()
        .filter_map(|(index, token)| {
            (!matches!(token.token, Token::Whitespace(_))).then_some(index)
        })
        .collect()
}
