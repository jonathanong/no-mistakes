use super::*;

#[test]
fn tsconfig_visible_virtual_modules_prove_declared_bases_only() {
    let source = fixture()
        .parent()
        .unwrap()
        .join("virtual-module-types/fixture");
    // A private copy keeps parser counts from colliding with the parity test.
    let saved = crate::test_support::materialize_saved_fixture(&source);
    let root = crate::codebase::ts_resolver::normalize_path(saved.path());
    // An `extends` path without a json suffix must load inherited `types`. The included
    // declaration is also a `types` entry and must not be parsed again.
    let observer = crate::diagnostics::InvocationObserver::new(true);
    crate::ast::begin_parse_count(&root);
    let report = {
        let _guard = crate::diagnostics::InvocationGuard::install(observer.clone());
        diagnostic_report(&root)
    };
    let parses = crate::ast::finish_parse_count(&root);
    assert_eq!(
        reported_channel(&report, "rules"),
        [
            "src/generated.ts:3 UnusedGenerated",
            "src/local.ts:3 UnusedLocal",
            "src/runtime.ts:4 UnusedRuntime",
            "src/runtime.ts:8 UnusedDirect",
        ],
        "{report}"
    );
    assert_eq!(
        reported_channel(&report, "advisories"),
        [
            "src/ambiguous.ts:2 unanalyzable",
            "src/missing.ts:2 unanalyzable",
        ],
        "{report}"
    );
    let local = root.join("types/local-export.d.ts");
    assert_eq!(parses.get(&local), Some(&1), "{parses:#?}");
    assert!(parses.values().all(|count| *count == 1), "{parses:#?}");
    let work = observer.snapshot().work;
    // Supplemental collection is only the two `types` entries outside the
    // analyzed set. The included declaration stays on the project fact pass.
    assert_eq!(work["ts_facts.collections"], 2, "{work:#?}");
    assert_eq!(work["ts_facts.files"], 2, "{work:#?}");
    assert_eq!(work["check.fact_collections"], 1, "{work:#?}");
}
