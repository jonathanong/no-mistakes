use super::run_all;
use std::path::PathBuf;
use std::sync::Arc;

fn check(config: &str) -> (serde_json::Value, u64, u64, u64) {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres/prepared-projection-scope");
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = fixture.path().canonicalize().unwrap();
    let observer = no_mistakes::diagnostics::InvocationObserver::new(true);
    let result = {
        let _guard = no_mistakes::diagnostics::InvocationGuard::install(Arc::clone(&observer));
        run_all(root.clone(), Some(root.join(config)), None).unwrap()
    };
    let work = observer.snapshot().work;
    (
        crate::check_runner::json_value(&result),
        work["postgres.prepared.statement_projections"],
        work["postgres.prepared.fragment_projections"],
        work["postgres.prepared.statement_records"],
    )
}

#[test]
fn aggregate_postgres_projections_follow_profile_scan_union() {
    let (narrow, narrow_statements, narrow_fragments, narrow_records) =
        check(".no-mistakes.narrow.yml");
    let (mixed, mixed_statements, mixed_fragments, mixed_records) = check(".no-mistakes.mixed.yml");
    let (broad, broad_statements, broad_fragments, broad_records) = check(".no-mistakes.broad.yml");
    // Clean out-of-scope calls still contain SQL, so equal reports alone cannot
    // protect against preparing every profile for every file again.
    assert_eq!(narrow["rules"].as_array().unwrap().len(), 6, "{narrow}");
    assert_eq!(narrow, mixed);
    assert_eq!(narrow, broad);
    assert_eq!((narrow_statements, narrow_fragments), (2, 2));
    assert_eq!((mixed_statements, mixed_fragments), (4, 4));
    assert_eq!((broad_statements, broad_fragments), (8, 8));
    // Each matching executor has one concrete call with no variants. The
    // broad control's inactive profiles contribute no statement records.
    assert_eq!((narrow_records, mixed_records, broad_records), (2, 4, 4));
}

fn low_check(config: &str) -> (serde_json::Value, u64, u64, u64) {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres/prepared-projection-low");
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = fixture.path().canonicalize().unwrap();
    let observer = no_mistakes::diagnostics::InvocationObserver::new(true);
    let result = {
        let _guard = no_mistakes::diagnostics::InvocationGuard::install(Arc::clone(&observer));
        crate::check_runner::run_all_with_sql_verdicts(
            root.clone(),
            Some(root.join(config)),
            None,
            false,
            false,
            true,
        )
        .unwrap()
    };
    let work = observer.snapshot().work;
    (
        crate::check_runner::json_value(&result),
        work["postgres.prepared.statement_projections"],
        work.get("postgres.no_offset.low.literal_parses")
            .copied()
            .unwrap_or(0),
        work["postgres.prepared.statement_records"],
    )
}

#[test]
fn aggregate_pruned_projections_preserve_low_offset_literal_fallback() {
    let (narrow, narrow_projections, narrow_literals, narrow_records) =
        low_check(".no-mistakes.narrow.yml");
    let (broad, broad_projections, broad_literals, broad_records) =
        low_check(".no-mistakes.broad.yml");
    assert_eq!(narrow, broad);
    assert_eq!(narrow["rules"].as_array().unwrap().len(), 2, "{narrow}");
    assert!(
        !narrow["advisories"].as_array().unwrap().is_empty(),
        "{narrow}"
    );
    assert!(
        !narrow["sqlVerdicts"].as_array().unwrap().is_empty(),
        "{narrow}"
    );
    assert_eq!((narrow_projections, narrow_literals), (1, 2));
    assert_eq!((broad_projections, broad_literals), (3, 0));
    assert_eq!((narrow_records, broad_records), (1, 3));
    // Pruning must not increase total SQL preparation by moving work into the
    // low-effort literal parser; both controls perform three preparations.
    assert_eq!(
        narrow_records + narrow_literals,
        broad_records + broad_literals
    );
}
