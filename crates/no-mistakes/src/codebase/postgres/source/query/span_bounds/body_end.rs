use super::*;
use sqlparser::ast::LimitClause;

impl QuerySpanBounds {
    /// The body excludes query-level clauses; their AST starts disambiguate
    /// keywords used as aliases or qualified object names inside a branch.
    pub(in crate::codebase::postgres::source::query) fn body_end(
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
            let offset_only = matches!(
                value,
                LimitClause::LimitOffset {
                    limit: None,
                    offset: Some(_),
                    ..
                }
            );
            let indexes = if offset_only {
                &self.offsets
            } else {
                &self.limits
            };
            let anchor = if matches!(
                value,
                LimitClause::LimitOffset {
                    limit: None,
                    offset: None,
                    ..
                }
            ) {
                end
            } else {
                locations.position(value.span().start)?.offset
            };
            let start = self.clause_before(indexes, anchor, lower)?;
            // sqlparser represents LIMIT ALL OFFSET n like an offset-only
            // clause. Include the adjacent LIMIT ALL prefix in the suffix.
            if offset_only {
                let offset_index = self.tokens.partition_point(|token| token.start < start);
                if offset_index >= 2
                    && self.tokens[offset_index - 1].keyword == Keyword::ALL
                    && self.tokens[offset_index - 2].keyword == Keyword::LIMIT
                    && self.tokens[offset_index - 2].start >= lower
                {
                    return Some(self.tokens[offset_index - 2].start);
                }
            }
            Some(start)
        });
        let fetch = query.fetch.as_ref().and_then(|value| {
            let anchor = if value.quantity.is_none() {
                end
            } else {
                locations.position(value.span().start)?.offset
            };
            self.clause_before(&self.fetches, anchor, lower)
        });
        // sqlparser drops a standalone LIMIT ALL from the AST entirely.
        let implicit_limit_all = if query.limit_clause.is_none() {
            let before = self.tokens.partition_point(|token| token.end <= end);
            self.tokens
                .get(before.checked_sub(1)?)
                .filter(|token| token.keyword == Keyword::ALL)
                .and_then(|_| self.tokens.get(before.checked_sub(2)?))
                .filter(|token| token.keyword == Keyword::LIMIT && token.start >= lower)
                .map(|token| token.start)
        } else {
            None
        };
        if (query.order_by.is_some() && order.is_none())
            || (query.limit_clause.is_some() && limit.is_none())
            || (query.fetch.is_some() && fetch.is_none())
        {
            return None;
        }
        let first_suffix = [order, limit, fetch, implicit_limit_all]
            .into_iter()
            .flatten()
            .min();
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

    pub(super) fn clause_before(&self, indexes: &[usize], anchor: usize, lower: usize) -> Option<usize> {
        let before = indexes.partition_point(|&index| self.tokens[index].start <= anchor);
        let start = self.tokens[*indexes.get(before.checked_sub(1)?)?].start;
        (start >= lower).then_some(start)
    }
}
