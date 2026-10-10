//! Query spans from sqlparser can stop before a trailing function's parentheses.
//! Use the prepared source tokens to find the enclosing query delimiter once.

use super::super::{locations::Locations, types::PostgresSqlSpan};
use sqlparser::ast::{Query, Spanned};
use sqlparser::keywords::Keyword;
use sqlparser::tokenizer::{Token, TokenWithSpan};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Open,
    Close,
    Other,
}

struct BoundToken {
    start: usize,
    end: usize,
    kind: Kind,
    keyword: Keyword,
    closing: Option<usize>,
}

pub(super) struct QuerySpanBounds {
    tokens: Vec<BoundToken>,
    set_operators: Vec<usize>,
    order_by: Vec<usize>,
    limits: Vec<usize>,
    offsets: Vec<usize>,
    fetches: Vec<usize>,
}

impl QuerySpanBounds {
    pub(super) fn new(tokens: &[TokenWithSpan], locations: &Locations<'_>) -> Self {
        let mut significant = Vec::<BoundToken>::new();
        let mut openings = Vec::<usize>::new();
        let mut set_operators = Vec::<usize>::new();
        let mut order_by = Vec::<usize>::new();
        let mut limits = Vec::<usize>::new();
        let mut offsets = Vec::<usize>::new();
        let mut fetches = Vec::<usize>::new();
        for token in tokens {
            if matches!(
                token.token,
                Token::Whitespace(_) | Token::EOF | Token::SemiColon
            ) {
                continue;
            }
            let (Some(start), Some(end)) = (
                locations.position(token.span.start),
                locations.position(token.span.end),
            ) else {
                continue;
            };
            let index = significant.len();
            if token.token == Token::RParen {
                if let Some(open) = openings.pop() {
                    significant[open].closing = Some(index);
                }
            }
            let kind = if token.token == Token::LParen {
                openings.push(index);
                Kind::Open
            } else if token.token == Token::RParen {
                Kind::Close
            } else {
                Kind::Other
            };
            let keyword = match &token.token {
                Token::Word(word) if word.quote_style.is_none() => word.keyword,
                _ => Keyword::NoKeyword,
            };
            if keyword == Keyword::BY
                && significant
                    .last()
                    .is_some_and(|token| token.keyword == Keyword::ORDER)
            {
                order_by.push(index - 1);
            }
            match keyword {
                Keyword::LIMIT => limits.push(index),
                Keyword::OFFSET => offsets.push(index),
                Keyword::FETCH => fetches.push(index),
                Keyword::UNION | Keyword::INTERSECT | Keyword::EXCEPT => set_operators.push(index),
                _ => {}
            }
            significant.push(BoundToken {
                start: start.offset,
                end: end.offset,
                kind,
                keyword,
                closing: None,
            });
        }
        Self {
            tokens: significant,
            set_operators,
            order_by,
            limits,
            offsets,
            fetches,
        }
    }

    pub(super) fn query(
        &self,
        query: &Query,
        root: bool,
        locations: &Locations<'_>,
    ) -> Option<PostgresSqlSpan> {
        let start = locations.position(query.span().start)?.offset;
        let first = self.tokens.first()?;
        let index = self.tokens.partition_point(|token| token.start < start);
        // A root's leading parentheses can wrap only its body; ORDER BY may
        // follow them. Keep all wrappers when sqlparser starts at the body.
        let (span_start, end) = if root
            && (first.start == start
                || (index > 0
                    && self.tokens[..index]
                        .iter()
                        .all(|token| token.kind == Kind::Open)))
        {
            (first.start, self.tokens[self.tokens.len() - 1].end)
        } else {
            let opening = self.tokens.get(index.checked_sub(1)?)?;
            if opening.kind != Kind::Open {
                return None;
            }
            let closing = opening.closing?;
            // A paired close always follows its opening token.
            (start, self.tokens[closing - 1].end)
        };
        (end >= span_start).then(|| locations.range(span_start, end))
    }
}

mod body_end;
mod insert_source;
mod set_branches;

#[cfg(test)]
mod tests;
