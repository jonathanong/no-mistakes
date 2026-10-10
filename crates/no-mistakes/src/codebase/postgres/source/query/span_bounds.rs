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
            } else if token.token == Token::RParen {
                Kind::Close
            } else {
                Kind::Other
            };
            let keyword = match &token.token {
                Token::Word(word) if word.quote_style.is_none() => word.keyword,
                _ => Keyword::NoKeyword,
            };
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

    /// A bare INSERT source has no opening delimiter; use its owning CTE boundary.
    pub(super) fn insert_source(
        &self,
        query: &Query,
        owner_end: usize,
        on_conflict_cutoff: Option<usize>,
        returning_item_start: Option<usize>,
        locations: &Locations<'_>,
    ) -> Option<PostgresSqlSpan> {
        let start = locations.position(query.span().start)?.offset;
        let from = self.tokens.partition_point(|token| token.start < start);
        let mut depth = 0usize;
        let mut last_on_conflict = None;
        let mut last_returning = None;
        let mut until = from;
        for (offset, token) in self.tokens[from..].iter().enumerate() {
            if token.start >= owner_end {
                break;
            }
            let index = from + offset;
            until = index + 1;
            match token.kind {
                Kind::Open => depth += 1,
                Kind::Close => depth = depth.saturating_sub(1),
                Kind::Other => {}
            }
            if depth == 0 {
                if on_conflict_cutoff.is_some_and(|end| token.start < end)
                    && token.keyword == Keyword::ON
                    && self.tokens.get(index + 1).is_some_and(|next| {
                        next.keyword == Keyword::CONFLICT && next.start < owner_end
                    })
                {
                    last_on_conflict = Some(index);
                }
                if returning_item_start.is_some_and(|start| token.start < start)
                    && token.keyword == Keyword::RETURNING
                {
                    last_returning = Some(index);
                }
            }
        }
        let boundary = [last_on_conflict, last_returning]
            .into_iter()
            .flatten()
            .min()
            .unwrap_or(until);
        let end = self.tokens.get(from..boundary)?.last()?.end;
        (end >= start).then(|| locations.range(start, end))
    }
}

#[cfg(test)]
mod tests;
