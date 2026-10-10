//! Query spans from sqlparser can stop before a trailing function's parentheses.
//! Use the prepared source tokens to find the enclosing query delimiter once.

use super::super::{locations::Locations, types::PostgresSqlSpan};
use sqlparser::ast::{LimitClause, Query, Spanned};
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

    /// The body excludes query-level clauses; their AST starts disambiguate
    /// keywords used as aliases or qualified object names inside a branch.
    pub(super) fn body_end(
        &self,
        query: &Query,
        span: Option<&PostgresSqlSpan>,
        locations: &Locations<'_>,
    ) -> Option<usize> {
        let end = span?.end.offset;
        // sqlparser does not retain the source span of a lock clause. Keep the
        // existing parser-span fallback when a query has one.
        if !query.locks.is_empty() {
            return None;
        }
        let lower = locations.position(query.body.span().start)?.offset;
        let order = query.order_by.as_ref().and_then(|value| {
            let anchor = locations.position(value.span().start)?.offset;
            self.clause_before(&self.order_by, anchor, lower)
        });
        let limit = query.limit_clause.as_ref().and_then(|value| {
            let anchor = locations.position(value.span().start)?.offset;
            let indexes = match value {
                LimitClause::LimitOffset { limit: None, .. } => &self.offsets,
                _ => &self.limits,
            };
            self.clause_before(indexes, anchor, lower)
        });
        let fetch = query.fetch.as_ref().and_then(|value| {
            let anchor = locations.position(value.span().start)?.offset;
            self.clause_before(&self.fetches, anchor, lower)
        });
        if (query.order_by.is_some() && order.is_none())
            || (query.limit_clause.is_some() && limit.is_none())
            || (query.fetch.is_some() && fetch.is_none())
        {
            return None;
        }
        let first_suffix = [order, limit, fetch].into_iter().flatten().min();
        let Some(suffix_start) = first_suffix else {
            return Some(end);
        };
        let before = self
            .tokens
            .partition_point(|token| token.start < suffix_start);
        self.tokens
            .get(before.checked_sub(1)?)
            .map(|token| token.end)
    }

    fn clause_before(&self, indexes: &[usize], anchor: usize, lower: usize) -> Option<usize> {
        let before = indexes.partition_point(|&index| self.tokens[index].start <= anchor);
        let start = self.tokens[*indexes.get(before.checked_sub(1)?)?].start;
        (start >= lower).then_some(start)
    }
}

mod insert_source;
mod set_branches;

#[cfg(test)]
mod tests;
