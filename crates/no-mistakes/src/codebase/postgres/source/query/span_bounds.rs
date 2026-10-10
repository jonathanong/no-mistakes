//! Query spans from sqlparser can stop before a trailing function's parentheses.
//! Use the prepared source tokens to find the enclosing query delimiter once.

use super::super::{locations::Locations, types::PostgresSqlSpan};
use sqlparser::ast::{Query, Spanned};
use sqlparser::tokenizer::{Token, TokenWithSpan};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Open,
    Other,
}

struct BoundToken {
    start: usize,
    end: usize,
    kind: Kind,
    closing: Option<usize>,
}

pub(super) struct QuerySpanBounds {
    tokens: Vec<BoundToken>,
}

impl QuerySpanBounds {
    pub(super) fn new(tokens: &[TokenWithSpan], locations: &Locations<'_>) -> Self {
        let mut significant = Vec::<BoundToken>::new();
        let mut openings = Vec::<usize>::new();
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
            } else {
                Kind::Other
            };
            significant.push(BoundToken {
                start: start.offset,
                end: end.offset,
                kind,
                closing: None,
            });
        }
        Self {
            tokens: significant,
        }
    }

    pub(super) fn query(
        &self,
        query: &Query,
        root: bool,
        locations: &Locations<'_>,
    ) -> Option<PostgresSqlSpan> {
        let start = locations.position(query.span().start)?.offset;
        let index = self.tokens.partition_point(|token| token.start < start);
        // A root's leading `(` can wrap only its body; ORDER BY may follow it.
        let (span_start, end) = if root
            && (self.tokens.first()?.start == start
                || (self.tokens.first()?.kind == Kind::Open && self.tokens.get(1)?.start == start))
        {
            (self.tokens.first()?.start, self.tokens.last()?.end)
        } else {
            let opening = self.tokens.get(index.checked_sub(1)?)?;
            if opening.kind != Kind::Open {
                return None;
            }
            let closing = opening.closing?;
            (start, self.tokens.get(closing.checked_sub(1)?)?.end)
        };
        (end >= span_start).then(|| locations.range(span_start, end))
    }
}

#[cfg(test)]
mod tests;
