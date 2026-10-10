use super::*;
use sqlparser::ast::SetExpr;

impl QuerySpanBounds {
    pub(in crate::codebase::postgres::source::query) fn set_branches(
        &self,
        left: &SetExpr,
        right: &SetExpr,
        parent: Option<&PostgresSqlSpan>,
        body_end: Option<usize>,
        locations: &Locations<'_>,
    ) -> (Option<PostgresSqlSpan>, Option<PostgresSqlSpan>) {
        let Some((parent, body_end, left_start, right_start)) = (|| {
            Some((
                parent?,
                body_end?,
                locations.position(left.span().start)?.offset,
                locations.position(right.span().start)?.offset,
            ))
        })() else {
            return (None, None);
        };
        // Every recursive set node finds its own separator with a binary search
        // in the request's token index, so long left-associated chains do not
        // rescan an ever-growing prefix.
        let before = self
            .set_operators
            .partition_point(|&index| self.tokens[index].start < right_start);
        let Some(separator) = before
            .checked_sub(1)
            .and_then(|index| self.set_operators.get(index))
            .and_then(|&index| self.tokens.get(index).map(|token| (index, token)))
            .filter(|(_, token)| token.start >= left_start)
        else {
            return (None, None);
        };
        let Some(left_end) = separator
            .0
            .checked_sub(1)
            .and_then(|index| self.tokens.get(index))
            .map(|token| token.end)
        else {
            return (None, None);
        };
        let left_start = self.branch_start(left, left_start, parent.start.offset, left_end);
        let right_start = self.branch_start(right, right_start, separator.1.end, body_end);
        if left_end < left_start || body_end < right_start {
            return (None, None);
        }
        (
            Some(locations.range(left_start, left_end)),
            Some(locations.range(right_start, body_end)),
        )
    }

    fn branch_start(
        &self,
        branch: &SetExpr,
        start: usize,
        lower_bound: usize,
        end: usize,
    ) -> usize {
        let mut start = if matches!(branch, SetExpr::Values(_)) {
            let before = self.tokens.partition_point(|token| token.start < start);
            before
                .checked_sub(1)
                .and_then(|index| self.tokens.get(index))
                .filter(|token| token.keyword == Keyword::VALUES && token.start < end)
                .map(|token| token.start)
                .unwrap_or(start)
        } else {
            start
        };
        loop {
            let index = self.tokens.partition_point(|token| token.start < start);
            let opening = index
                .checked_sub(1)
                .and_then(|index| self.tokens.get(index));
            let Some(opening) = opening.filter(|token| token.kind == Kind::Open) else {
                return start;
            };
            let closes_inside = opening
                .closing
                .and_then(|index| self.tokens.get(index))
                .is_some_and(|closing| closing.end <= end);
            if opening.start < lower_bound || !closes_inside {
                return start;
            }
            start = opening.start;
        }
    }
}
