use super::*;

impl QuerySpanBounds {
    /// A bare INSERT source has no opening delimiter; use its owning CTE boundary.
    pub(in crate::codebase::postgres::source::query) fn insert_source(
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
