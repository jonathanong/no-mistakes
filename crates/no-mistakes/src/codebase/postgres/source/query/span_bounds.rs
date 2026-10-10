//! Query spans from sqlparser can stop before a trailing function's parentheses.
//! Use the prepared source tokens to find the enclosing query delimiter once.

use super::super::{locations::Locations, types::PostgresSqlSpan};
use sqlparser::ast::{Query, SetExpr, Spanned};
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
    set_operator: bool,
    query_suffix: bool,
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
            let word = match &token.token {
                Token::Word(word) if word.quote_style.is_none() => Some(word.value.as_str()),
                _ => None,
            };
            significant.push(BoundToken {
                start: start.offset,
                end: end.offset,
                kind,
                closing: None,
                set_operator: word.is_some_and(|word| {
                    ["UNION", "INTERSECT", "EXCEPT"]
                        .iter()
                        .any(|keyword| word.eq_ignore_ascii_case(keyword))
                }),
                query_suffix: word.is_some_and(|word| {
                    ["ORDER", "LIMIT", "OFFSET", "FETCH", "FOR"]
                        .iter()
                        .any(|keyword| word.eq_ignore_ascii_case(keyword))
                }),
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
        let end = if root && self.tokens.first()?.start == start {
            self.tokens.last()?.end
        } else {
            let opening = self.tokens.get(index.checked_sub(1)?)?;
            if opening.kind != Kind::Open {
                return None;
            }
            let closing = opening.closing?;
            self.tokens.get(closing.checked_sub(1)?)?.end
        };
        (end >= start).then(|| locations.range(start, end))
    }

    pub(super) fn set_branches(
        &self,
        left: &SetExpr,
        right: &SetExpr,
        parent: Option<&PostgresSqlSpan>,
        locations: &Locations<'_>,
    ) -> (Option<PostgresSqlSpan>, Option<PostgresSqlSpan>) {
        let Some((left_start, right_start, parent_end)) = parent.and_then(|parent| {
            Some((
                locations.position(left.span().start)?.offset,
                locations.position(right.span().start)?.offset,
                parent.end.offset,
            ))
        }) else {
            return (None, None);
        };
        let mut index = self
            .tokens
            .partition_point(|token| token.start < left_start);
        let mut separator = None;
        while let Some(token) = self.tokens.get(index) {
            if token.start >= right_start {
                break;
            }
            if token.kind == Kind::Open {
                if let Some(close) = token.closing.filter(|close| *close < self.tokens.len()) {
                    index = close + 1;
                    continue;
                }
            }
            if token.set_operator {
                separator = Some(index);
            }
            index += 1;
        }
        let left_span = separator
            .and_then(|index| self.tokens.get(index.checked_sub(1)?))
            .filter(|token| token.end >= left_start)
            .map(|token| locations.range(left_start, token.end));

        let mut index = self
            .tokens
            .partition_point(|token| token.start < right_start);
        let mut right_end = None;
        while let Some(token) = self.tokens.get(index) {
            if token.start >= parent_end || token.query_suffix {
                break;
            }
            if token.kind == Kind::Open {
                if let Some((close, closing)) = token.closing.and_then(|close| {
                    self.tokens
                        .get(close)
                        .filter(|closing| closing.start < parent_end)
                        .map(|closing| (close, closing))
                }) {
                    right_end = Some(closing.end);
                    index = close + 1;
                    continue;
                }
            }
            right_end = Some(token.end);
            index += 1;
        }
        let right_span = right_end
            .filter(|end| *end >= right_start)
            .map(|end| locations.range(right_start, end));
        (left_span, right_span)
    }
}

#[cfg(test)]
mod tests;
