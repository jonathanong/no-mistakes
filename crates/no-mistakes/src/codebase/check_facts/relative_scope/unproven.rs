use super::super::CheckFileFacts;
use std::collections::BTreeSet;

pub fn unproven_member_query_warning(count: u32) -> Option<String> {
    (count > 0).then(|| {
        format!(
            "warning: {count} member .query calls with SQL-looking text weren't proven to be executors; see docs/migrations/explicit-postgres-executors.md"
        )
    })
}

pub(crate) fn count_unproven_member_queries<'a>(
    files: impl IntoIterator<Item = &'a CheckFileFacts>,
) -> u32 {
    let mut total = 0u32;
    for facts in files {
        let mut spans = BTreeSet::new();
        for (_, embedded) in &facts.embedded_sql {
            spans.extend(embedded.unproven_member_query_spans.iter().copied());
        }
        total = total.saturating_add(spans.len() as u32);
    }
    total
}

pub(super) fn note_unproven_member_queries(count: u32) {
    if count == 0 {
        return;
    }
    if let Some(observer) = crate::diagnostics::current().filter(|observer| observer.verbose()) {
        observer.increment("postgres.unproven_member_query_sql", u64::from(count));
    }
}
