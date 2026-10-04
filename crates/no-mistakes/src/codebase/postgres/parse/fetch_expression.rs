//! Parse expression counts through LIMIT, then restore the original FETCH AST.
use sqlparser::{
    ast::{Fetch, LimitClause, Query, Spanned, VisitMut, VisitorMut},
    keywords::Keyword,
    tokenizer::{Location, Span, Token, TokenWithSpan, Whitespace, Word},
};
use std::ops::ControlFlow;

pub(in crate::codebase::postgres) struct Clause {
    start: Location,
    end: Location,
    expression: Span,
    with_ties: bool,
}

pub(in crate::codebase::postgres) fn prepare(tokens: &mut [TokenWithSpan]) -> Vec<Clause> {
    // FETCH expressions are uncommon; avoid allocating the significant-token list and
    // parenthesis index for the overwhelmingly common statement with no FETCH keyword.
    if !tokens
        .iter()
        .any(|token| keyword(&token.token, Keyword::FETCH))
    {
        return Vec::new();
    }
    let significant = tokens
        .iter()
        .enumerate()
        .filter_map(|(index, token)| {
            (!matches!(token.token, Token::Whitespace(_))).then_some(index)
        })
        .collect::<Vec<_>>();
    // One balanced-parenthesis index avoids rescanning nested count expressions.
    let mut closing = vec![None; significant.len()];
    let mut stack = Vec::new();
    for (position, index) in significant.iter().copied().enumerate() {
        match tokens[index].token {
            Token::LParen => stack.push(position),
            Token::RParen => {
                if let Some(open) = stack.pop() {
                    closing[open] = Some(position);
                }
            }
            _ => {}
        }
    }
    let mut clauses = Vec::new();
    for (position, index) in significant.iter().copied().enumerate() {
        if !keyword(&tokens[index].token, Keyword::FETCH) {
            continue;
        }
        let Some(&direction) = significant.get(position + 1) else {
            continue;
        };
        if !keyword(&tokens[direction].token, Keyword::FIRST)
            && !keyword(&tokens[direction].token, Keyword::NEXT)
        {
            continue;
        }
        let quantity = position + 2;
        let Some(&open) = significant.get(quantity) else {
            continue;
        };
        if tokens[open].token != Token::LParen {
            continue;
        }
        let Some(close) = closing[quantity] else {
            continue;
        };
        let Some(&rows) = significant.get(close + 1) else {
            continue;
        };
        if !keyword(&tokens[rows].token, Keyword::ROW)
            && !keyword(&tokens[rows].token, Keyword::ROWS)
        {
            continue;
        }
        let Some(&extension) = significant.get(close + 2) else {
            continue;
        };
        let with_ties = keyword(&tokens[extension].token, Keyword::WITH)
            && significant
                .get(close + 3)
                .is_some_and(|i| keyword(&tokens[*i].token, Keyword::TIES));
        if !with_ties && !keyword(&tokens[extension].token, Keyword::ONLY) {
            continue;
        }
        clauses.push(Clause {
            start: tokens[index].span.start,
            end: tokens[significant[close + 2 + usize::from(with_ties)]]
                .span
                .end,
            expression: Span {
                start: tokens[open].span.start,
                end: tokens[significant[close]].span.end,
            },
            with_ties,
        });
        tokens[index].token = Token::Word(Word {
            value: "LIMIT".into(),
            quote_style: None,
            keyword: Keyword::LIMIT,
        });
        for &i in &significant[position + 1..quantity] {
            tokens[i].token = Token::Whitespace(Whitespace::Space);
        }
        for &i in &significant[close + 1..=close + 2 + usize::from(with_ties)] {
            tokens[i].token = Token::Whitespace(Whitespace::Space);
        }
    }
    clauses
}

pub(in crate::codebase::postgres) fn restore<T: VisitMut>(tree: &mut T, clauses: &[Clause]) {
    if clauses.is_empty() {
        return;
    }
    struct Restore<'a>(&'a [Clause]);
    impl VisitorMut for Restore<'_> {
        type Break = ();
        fn pre_visit_query(&mut self, query: &mut Query) -> ControlFlow<()> {
            let Some(LimitClause::LimitOffset {
                limit: Some(quantity),
                offset: _,
                limit_by: _,
            }) = &query.limit_clause
            else {
                return ControlFlow::Continue(());
            };
            let span = quantity.span();
            // The body precedes its FETCH. A nested LIMIT inside the count starts
            // after FETCH and must never inherit the enclosing clause's identity.
            let next = self
                .0
                .partition_point(|clause| clause.expression.start <= span.start);
            let Some(clause) = next.checked_sub(1).and_then(|index| self.0.get(index)) else {
                return ControlFlow::Continue(());
            };
            if query.body.span().end > clause.start || span.end > clause.expression.end {
                return ControlFlow::Continue(());
            }
            // The variant was matched above; take only its owned quantity.
            if let Some(LimitClause::LimitOffset { limit, offset, .. }) = &mut query.limit_clause {
                query.fetch = Some(Fetch {
                    with_ties: clause.with_ties,
                    percent: false,
                    quantity: limit.take(),
                });
                if offset.is_none() {
                    query.limit_clause = None;
                }
            }
            ControlFlow::Continue(())
        }
    }
    let _ = tree.visit(&mut Restore(clauses));
}

fn keyword(token: &Token, value: Keyword) -> bool {
    matches!(token, Token::Word(word) if word.quote_style.is_none() && word.keyword == value)
}

/// Removed FETCH suffixes still belong to the public original-source statement.
pub(in crate::codebase::postgres) fn source_end(
    clauses: &[Clause],
    start: Location,
    end: Location,
) -> Location {
    let first = clauses.partition_point(|clause| clause.start < start);
    clauses[first..]
        .iter()
        .take_while(|clause| clause.start <= end)
        .fold(end, |end, clause| end.max(clause.end))
}
