//! One pass marks non-SQL body offsets, then each token is tested in constant time.
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
    let marks = non_sql_marks(body.start, len, occurrences);
    let count = tokens.len();
    tokens.retain(|token| {
        let start = body_offset(&local, body, token.span.start);
        let end = body_offset(&local, body, token.span.end);
        !marks.overlaps(start, end)
    });
    debug_assert_eq!(marks.span_passes(), occurrences.len());
    debug_assert_eq!(marks.body_passes(), len);
    debug_assert_eq!(marks.token_checks(), count);
}

pub(super) fn sql_occurrence(kind: PostgresSqlProceduralOccurrenceKind) -> bool {
    matches!(
        kind,
        PostgresSqlProceduralOccurrenceKind::Utility | PostgresSqlProceduralOccurrenceKind::Dml
    )
}

pub(super) struct NonSqlMarks {
    origin: usize,
    len: usize,
    /// Exclusive prefix of body offsets covered by a non-SQL span.
    prefix: Vec<usize>,
    span_passes: usize,
    body_passes: usize,
    token_checks: Cell<usize>,
}

impl NonSqlMarks {
    /// Half-open overlap with the marked union: `start < span_end && span_start < end`.
    pub(super) fn overlaps(&self, start: usize, end: usize) -> bool {
        self.token_checks.set(self.token_checks.get() + 1);
        let from = self.index(start);
        let to = self.index(end);
        from < to && self.prefix[to] > self.prefix[from]
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

    fn index(&self, offset: usize) -> usize {
        offset.saturating_sub(self.origin).min(self.len)
    }
}

/// Marks the union of non-SQL spans in `[origin, origin + len)`.
///
/// Only this slice is visited. Nested children stay inside their parent span.
pub(super) fn non_sql_marks(
    origin: usize,
    len: usize,
    occurrences: &[PostgresSqlProceduralOccurrence],
) -> NonSqlMarks {
    // A difference sweep paints the union once. Filling each span in place would
    // rescan overlapping bytes and stop being linear in spans plus body length.
    let mut diff = vec![0isize; len + 1];
    let mut span_passes = 0;
    for occurrence in occurrences {
        span_passes += 1;
        if sql_occurrence(occurrence.kind) {
            continue;
        }
        let from = occurrence.span.start.offset.saturating_sub(origin).min(len);
        let to = occurrence.span.end.offset.saturating_sub(origin).min(len);
        if from < to {
            diff[from] += 1;
            diff[to] -= 1;
        }
    }
    let mut marked = vec![false; len];
    let mut prefix = vec![0usize; len + 1];
    let mut open = 0isize;
    let mut body_passes = 0;
    for (index, cell) in marked.iter_mut().enumerate() {
        body_passes += 1;
        open += diff[index];
        *cell = open > 0;
        prefix[index + 1] = prefix[index] + usize::from(*cell);
    }
    NonSqlMarks {
        origin,
        len,
        prefix,
        span_passes,
        body_passes,
        token_checks: Cell::new(0),
    }
}

fn body_offset(local: &Locations<'_>, body: &Body<'_>, location: Location) -> usize {
    let position = local.position(location).expect("procedural token");
    body.offset(position.offset).expect("procedural token")
}
