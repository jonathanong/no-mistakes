//! Non-SQL spans are merged intervals. A long literal does not allocate per byte.
pub(super) mod label;

use super::body::Body;
use super::locations::Locations;
use super::types::{PostgresSqlProceduralOccurrence, PostgresSqlProceduralOccurrenceKind};
use sqlparser::tokenizer::{Location, TokenWithSpan};
use std::cell::Cell;

pub(super) fn omit_non_sql(
    tokens: &mut Vec<TokenWithSpan>,
    body: &Body<'_>,
    occurrences: &[PostgresSqlProceduralOccurrence],
) {
    let local = Locations::new(body.sql.as_ref());
    let len = body.end.saturating_sub(body.start);
    let mut marks = non_sql_marks(body.start, len, occurrences);
    // The walker skips `<<label>>` before the span. Extend the omitted interval only.
    for (start, end) in label::opening_labels(body, occurrences, tokens, &local) {
        marks.cover(start, end);
    }
    let count = tokens.len();
    tokens.retain(|token| {
        let start = body_offset(&local, body, token.span.start);
        let end = body_offset(&local, body, token.span.end);
        !marks.overlaps(start, end)
    });
    debug_assert_eq!(marks.span_passes(), occurrences.len());
    debug_assert!(marks.body_passes() <= occurrences.len());
    debug_assert_eq!(marks.token_checks(), count);
}

pub(super) fn sql_occurrence(kind: PostgresSqlProceduralOccurrenceKind) -> bool {
    matches!(
        kind,
        PostgresSqlProceduralOccurrenceKind::Utility | PostgresSqlProceduralOccurrenceKind::Dml
    )
}

struct Interval {
    start: usize,
    end: usize,
}

pub(super) struct NonSqlMarks {
    origin: usize,
    len: usize,
    /// Disjoint half-open intervals inside the body window, ordered by start.
    intervals: Vec<Interval>,
    span_passes: usize,
    body_passes: usize,
    token_checks: Cell<usize>,
}

impl NonSqlMarks {
    /// Half-open overlap with the marked union: `start < span_end && span_start < end`.
    pub(super) fn overlaps(&self, start: usize, end: usize) -> bool {
        self.token_checks.set(self.token_checks.get() + 1);
        let Some((start, end)) = clip(self.origin, self.len, start, end) else {
            return false;
        };
        let index = self
            .intervals
            .partition_point(|interval| interval.start < end);
        index > 0 && start < self.intervals[index - 1].end
    }

    pub(super) fn span_passes(&self) -> usize {
        self.span_passes
    }

    pub(super) fn body_passes(&self) -> usize {
        self.body_passes
    }

    pub(super) fn token_checks(&self) -> usize {
        self.token_checks.get()
    }

    /// Merge one more omitted range. Does not count as another occurrence.
    pub(super) fn cover(&mut self, start: usize, end: usize) {
        let window_end = self.origin.saturating_add(self.len);
        if start >= end || start < self.origin || end > window_end {
            return;
        }
        self.intervals.push(Interval { start, end });
        self.intervals = merge_intervals(std::mem::take(&mut self.intervals));
    }
}

/// Marks the union of non-SQL spans in `[origin, origin + len)`.
///
/// Only this slice is visited. Nested children stay inside their parent span.
/// Memory follows the non-SQL spans, not the body byte length.
pub(super) fn non_sql_marks(
    origin: usize,
    len: usize,
    occurrences: &[PostgresSqlProceduralOccurrence],
) -> NonSqlMarks {
    let mut intervals = Vec::new();
    let mut span_passes = 0;
    let mut body_passes = 0;
    for occurrence in occurrences {
        span_passes += 1;
        if sql_occurrence(occurrence.kind) {
            continue;
        }
        body_passes += 1;
        if let Some((start, end)) = clip(
            origin,
            len,
            occurrence.span.start.offset,
            occurrence.span.end.offset,
        ) {
            intervals.push(Interval { start, end });
        }
    }
    NonSqlMarks {
        origin,
        len,
        intervals: merge_intervals(intervals),
        span_passes,
        body_passes,
        token_checks: Cell::new(0),
    }
}

fn merge_intervals(mut intervals: Vec<Interval>) -> Vec<Interval> {
    intervals.sort_by_key(|interval| (interval.start, interval.end));
    let mut merged: Vec<Interval> = Vec::new();
    for interval in intervals {
        if let Some(last) = merged.last_mut() {
            if interval.start <= last.end {
                last.end = last.end.max(interval.end);
                continue;
            }
        }
        merged.push(interval);
    }
    merged
}

fn clip(origin: usize, len: usize, start: usize, end: usize) -> Option<(usize, usize)> {
    let start = start.max(origin);
    let end = end.min(origin.saturating_add(len));
    (start < end).then_some((start, end))
}

fn body_offset(local: &Locations<'_>, body: &Body<'_>, location: Location) -> usize {
    let position = local.position(location).expect("procedural token");
    body.offset(position.offset).expect("procedural token")
}
