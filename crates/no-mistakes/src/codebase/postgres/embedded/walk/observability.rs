use super::super::{EmbeddedSqlCall, EmbeddedSqlKind};

pub(super) fn record(calls: &[EmbeddedSqlCall]) {
    if let Some(observer) = crate::diagnostics::current() {
        for call in calls {
            let composed = if call.variants.is_empty() {
                usize::from(call.kind == EmbeddedSqlKind::Composed)
            } else if call
                .variants
                .iter()
                .any(|value| !value.append_sites.is_empty())
            {
                call.variants.len()
            } else {
                0
            };
            observer.increment("postgres.sql.variants_composed", composed as u64);
            observer.increment(
                "postgres.sql.sites_truncated",
                u64::from(call.variants_truncated()),
            );
        }
    }
}
