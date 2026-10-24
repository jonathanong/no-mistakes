use super::run_all;
use std::path::PathBuf;
use std::sync::Arc;

fn fixture() -> tempfile::TempDir {
    let saved = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/performance/nextjs-raw-source-scope");
    crate::test_support::materialize_saved_fixture(&saved)
}

fn files(results: &super::CheckResults) -> Vec<String> {
    let mut files = results
        .rules
        .iter()
        .filter(|finding| finding.rule == "nextjs-no-api-routes")
        .map(|finding| finding.file.clone())
        .collect::<Vec<_>>();
    files.sort();
    files
}

#[test]
fn api_route_demand_preserves_suppressed_react_parse_errors_without_candidates() {
    let saved = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/check/react-analyze-suppressed-parse-error");
    let fixture = crate::test_support::materialize_saved_fixture(&saved);
    let root = fixture.path().canonicalize().unwrap();
    let react_only = run_all(root.clone(), Some(root.join(".no-mistakes.yml")), None).unwrap();
    let with_api = run_all(root.clone(), Some(root.join("with-api.yml")), None).unwrap();

    assert!(react_only.react.is_empty(), "{:?}", react_only.react);
    assert!(with_api.react.is_empty(), "{:?}", with_api.react);
    assert_eq!(react_only.warnings.len(), 1, "{:?}", react_only.warnings);
    assert!(react_only.warnings[0].contains("react check skipped: failed to parse"));
    assert!(with_api.queues.is_empty(), "{:?}", with_api.queues);
    assert!(
        with_api.integration.is_empty(),
        "{:?}",
        with_api.integration
    );
    assert!(with_api.codebase.is_empty(), "{:?}", with_api.codebase);
    assert!(with_api.rules.is_empty(), "{:?}", with_api.rules);
    // The frozen runner skips the suppressed parse error when this raw-source
    // consumer is enabled; preserve its successful, warning-free result.
    assert!(with_api.warnings.is_empty(), "{:?}", with_api.warnings);
    assert_eq!(with_api.analysis_warning_count, 0);
    assert!(with_api.analysis_fallbacks.is_empty());
    assert!(with_api.advisories.is_empty(), "{:?}", with_api.advisories);
}

#[test]
fn raw_api_route_requests_read_only_selected_route_candidates() {
    let fixture = fixture();
    let root = fixture.path().canonicalize().unwrap();
    let mut expected = [
        "apps/admin/app/api/route.ts",
        // Raw-source-only checking has no parser, so this candidate is reported.
        "apps/store/app/api/broken/route.ts",
        "apps/store/app/api/route.js",
        "apps/store/app/api/route.jsx",
        "apps/store/app/api/route.ts",
        "apps/store/app/api/route.tsx",
        "apps/store/app/api/route.mjs",
        "apps/store/app/api/route.mts",
        "apps/store/app/api/route.cjs",
        "apps/store/app/api/route.cts",
        "apps/store/pages/api/legacy.ts",
    ]
    .map(str::to_owned)
    .to_vec();
    expected.sort();
    for jobs in [1, 4] {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(jobs)
            .build()
            .unwrap();
        let observer = no_mistakes::diagnostics::InvocationObserver::new(true);
        let result = pool.install(|| {
            let _guard = no_mistakes::diagnostics::InvocationGuard::install(Arc::clone(&observer));
            run_all(root.clone(), Some(root.join(".no-mistakes.yml")), None).unwrap()
        });
        let work = observer.snapshot().work;

        assert_eq!(files(&result), expected, "jobs={jobs}: {:?}", result.rules);
        assert_eq!(
            work.get("source.reads"),
            Some(&14),
            "jobs={jobs}: {work:#?}"
        );
        assert_eq!(
            work.get("source.read_errors").copied().unwrap_or_default(),
            0
        );
        assert_eq!(work.get("parse.files").copied().unwrap_or_default(), 0);
        assert_eq!(
            work.get("parse.files_after_extract")
                .copied()
                .unwrap_or_default(),
            0
        );
        assert!(
            work.get("source.bytes").copied().unwrap_or_default() < 12_000,
            "route-only request read dense ordinary source: {work:#?}"
        );
        assert!(result.warnings.is_empty(), "{:?}", result.warnings);
        assert!(result.advisories.is_empty(), "{:?}", result.advisories);
        assert_eq!(result.analysis_warning_count, 0);
        assert!(result.analysis_fallbacks.is_empty());
    }
}

#[test]
fn mixed_route_and_source_fact_consumers_keep_each_others_work() {
    let fixture = fixture();
    let root = fixture.path().canonicalize().unwrap();
    let raw_observer = no_mistakes::diagnostics::InvocationObserver::new(true);
    let raw_result = {
        let _guard = no_mistakes::diagnostics::InvocationGuard::install(Arc::clone(&raw_observer));
        run_all(root.clone(), Some(root.join(".no-mistakes.yml")), None).unwrap()
    };
    let mixed_observer = no_mistakes::diagnostics::InvocationObserver::new(true);
    let mixed_result = {
        let _guard =
            no_mistakes::diagnostics::InvocationGuard::install(Arc::clone(&mixed_observer));
        run_all(root.clone(), Some(root.join("mixed.yml")), None).unwrap()
    };
    let mixed_work = mixed_observer.snapshot().work;

    let mut mixed_expected = files(&raw_result);
    mixed_expected.retain(|file| file != "apps/store/app/api/broken/route.ts");
    assert_eq!(files(&mixed_result), mixed_expected);
    assert!(
        mixed_result.rules.iter().any(|finding| {
            finding.rule == "nextjs-no-caching" && finding.file == "apps/store/src/cache.ts"
        }),
        "{:?}",
        mixed_result.rules
    );
    assert!(
        mixed_work.get("parse.files").copied().unwrap_or_default() > 0,
        "the source-fact consumer must retain parsing: {mixed_work:#?}"
    );
    assert_eq!(
        mixed_work.get("source.reads"),
        Some(&22),
        "mixed consumer must retain its full selected source scope: {mixed_work:#?}"
    );
    assert_eq!(mixed_work.get("parse.files"), Some(&20), "{mixed_work:#?}");
    assert_eq!(
        mixed_work.get("parse.files_after_extract"),
        Some(&20),
        "the mixed request must reuse extracted facts: {mixed_work:#?}"
    );
    assert!(raw_result.warnings.is_empty(), "{:?}", raw_result.warnings);
    assert!(
        raw_result.advisories.is_empty(),
        "{:?}",
        raw_result.advisories
    );
    assert!(
        mixed_result.warnings.is_empty(),
        "{:?}",
        mixed_result.warnings
    );
    assert_eq!(raw_result.analysis_warning_count, 0);
    assert_eq!(mixed_result.analysis_warning_count, 0);
    assert!(raw_result.analysis_fallbacks.is_empty());
    assert!(mixed_result.analysis_fallbacks.is_empty());
    assert!(
        mixed_result.advisories.is_empty(),
        "{:?}",
        mixed_result.advisories
    );
}
