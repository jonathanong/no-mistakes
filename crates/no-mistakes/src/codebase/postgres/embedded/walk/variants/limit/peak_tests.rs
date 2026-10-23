use std::cell::Cell;
use std::path::PathBuf;

thread_local! {
    static PEAK: Cell<Option<usize>> = const { Cell::new(None) };
}

pub(super) fn record(count: usize) {
    PEAK.with(|peak| {
        if let Some(prior) = peak.get() {
            peak.set(Some(prior.max(count)));
        }
    });
}

#[test]
fn actual_join_candidates_stay_bounded_before_variant_selection() {
    use crate::codebase::postgres::{extract_embedded_sql_from_source, EmbeddedSqlOptions};
    for (name, bound) in [
        ("variants-append-decisions.ts", 44),
        ("variants-append-conditional-suffix.ts", 66),
        ("variants-append-both-arms.ts", 44),
    ] {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/postgres-facts/embedded")
            .join(name);
        let source = std::fs::read_to_string(&path).unwrap();
        PEAK.with(|peak| peak.set(Some(0)));
        let file = extract_embedded_sql_from_source(
            &path,
            &source,
            &EmbeddedSqlOptions::configured("@example/db", &["read".into()]).with_trusted_sql_tags(
                &[crate::codebase::postgres::TrustedSqlTag {
                    module: "@example/db".into(),
                    name: "sql".into(),
                }],
            ),
        );
        let peak = PEAK.with(|peak| peak.replace(None).unwrap());
        assert!(file.calls.iter().any(|call| call.variants_truncated()));
        // At most21 decisions and three recovered suffix alternatives: each
        // boundary sees at most3*max(16,n+1), never all2^n SQL outcomes.
        assert!(
            (17..=bound).contains(&peak),
            "{name}: peak candidates {peak}"
        );
    }
}
