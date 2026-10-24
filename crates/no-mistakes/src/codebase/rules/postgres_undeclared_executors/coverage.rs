use crate::codebase::postgres::sql_identifier_syntax::{Input, SqlCall};
use crate::fx::FxHashMap;

// A canonical Unknown replaces only its own receiver obligation. Overlapping
// nested calls can belong to different executors and must retain both rows.
pub(super) fn unknown_ends(calls: &[SqlCall]) -> FxHashMap<u32, u32> {
    let mut ends = FxHashMap::default();
    for call in calls
        .iter()
        .filter(|call| matches!(call.input, Input::Unknown))
    {
        let previous = ends.entry(call.span.0).or_insert(call.span.1);
        *previous = (*previous).min(call.span.1);
    }
    ends
}
