//! Byte mapping for heads tokenized after the grouping compatibility edit.
use super::Edit;

#[derive(Default)]
pub(in crate::codebase::postgres::parse) struct HeadSourceMap(Vec<Rewrite>);

struct Rewrite {
    after_start: usize,
    after_end: usize,
    before_end: usize,
}

impl HeadSourceMap {
    pub(super) fn new(sql: &str, edits: &[Edit]) -> Self {
        let mut rewrites = Vec::with_capacity(edits.len());
        let mut previous_after = 0;
        let mut previous_before = 0;
        for edit in edits {
            let after_start = previous_after + edit.at - previous_before;
            let replaced = &sql[edit.at..edit.at + edit.replace];
            let after_end = after_start + 1 + replaced.chars().skip(1).count();
            let before_end = edit.at + edit.replace;
            rewrites.push(Rewrite {
                after_start,
                after_end,
                before_end,
            });
            previous_after = after_end;
            previous_before = before_end;
        }
        Self(rewrites)
    }

    /// Synthetic separators have no original lexeme; other spans map exactly.
    pub(in crate::codebase::postgres::parse) fn source_span(
        &self,
        start: usize,
        end: usize,
    ) -> Option<(usize, usize)> {
        let at = self.0.partition_point(|edit| edit.after_end <= start);
        if self.0.get(at).is_some_and(|edit| end > edit.after_start) {
            return None;
        }
        let Some(previous) = at.checked_sub(1).and_then(|at| self.0.get(at)) else {
            return Some((start, end));
        };
        Some((
            previous.before_end + start - previous.after_end,
            previous.before_end + end - previous.after_end,
        ))
    }
}
