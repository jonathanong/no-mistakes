use super::*;
use crate::cli::Format;
use crate::codebase::queries::render::render;
use std::path::PathBuf;
use std::sync::Arc;

fn fixture_root() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/queries/fixture"),
    )
}

fn test_plan_fixture_root(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/test-plan")
            .join(name),
    )
}

fn importers_fixture_root(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/importers")
            .join(name),
    )
}

fn args(file: &str, tests: bool) -> ImportersArgs {
    ImportersArgs {
        file: PathBuf::from(file),
        tests,
        root: Some(fixture_root()),
        tsconfig: None,
        format: None,
        json: false,
    }
}

#[test]
fn lists_direct_importers_and_count() {
    let json = run_json(args("util.ts", false)).unwrap();
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(
        value["directImporters"],
        serde_json::json!(["barrel.ts", "broken.ts", "consumer.ts"])
    );
    assert_eq!(value["dependentsCount"], 3);
    assert!(value.get("testImpact").is_none());
}

#[test]
fn reverse_query_discovers_and_parses_each_file_once() {
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let _guard = crate::diagnostics::InvocationGuard::install(Arc::clone(&observer));

    compute(&args("util.ts", false)).unwrap();

    let work = observer.snapshot().work;
    assert_eq!(work["discovery.roots"], 1, "{work:#?}");
    assert_eq!(work["symbol_index.builds"], 1, "{work:#?}");
    assert_eq!(work["parse.requests"], work["parse.files"], "{work:#?}");
}

#[test]
fn tests_flag_adds_impacted_tests() {
    let report = compute(&args("util.ts", true)).unwrap();
    assert_eq!(
        report.direct_importers,
        ["barrel.ts", "broken.ts", "consumer.ts"]
    );
    let impact = report.test_impact.expect("test impact present");
    assert_eq!(impact.count, 1);
    assert_eq!(impact.tests, vec!["consumer.test.ts".to_string()]);
}

#[test]
fn tests_flag_does_not_broaden_direct_importers_to_runner_scoped_aliases() {
    // The nested Vitest project owns this alias. It intentionally resolves
    // only for test impact, never for ordinary reverse-query output.
    let root = importers_fixture_root("tests-scoped-alias");
    let base = ImportersArgs {
        file: PathBuf::from("packages/unit/src/subject.ts"),
        tests: false,
        root: Some(root),
        tsconfig: None,
        format: None,
        json: false,
    };
    let ordinary = compute(&base).unwrap();
    assert!(ordinary.direct_importers.is_empty());
    assert_eq!(ordinary.dependents_count, 0);

    let with_tests = compute(&ImportersArgs {
        tests: true,
        ..base
    })
    .unwrap();
    assert_eq!(with_tests.direct_importers, ordinary.direct_importers);
    assert_eq!(with_tests.dependents_count, ordinary.dependents_count);
    let impact = with_tests.test_impact.expect("test impact present");
    assert_eq!(impact.tests, ["packages/unit/tests/owner.test.ts"]);
}

#[test]
fn tests_flag_keeps_recovered_imports_from_malformed_runner_helpers() {
    let fixture = crate::test_support::materialize_saved_fixture(&importers_fixture_root(
        "malformed-runner-helper",
    ));
    let root = fixture.path().canonicalize().unwrap();
    let base = ImportersArgs {
        file: PathBuf::from("src/subject.ts"),
        tests: false,
        root: Some(root.clone()),
        tsconfig: None,
        format: None,
        json: false,
    };
    let ordinary = compute(&base).unwrap();
    // `setup.ts` is parsed strictly while Vitest discovers setup closures,
    // but its recovered import must remain visible to ordinary importers.
    assert_eq!(ordinary.direct_importers, ["setup.ts"]);

    let observer = crate::diagnostics::InvocationObserver::new(true);
    let _guard = crate::diagnostics::InvocationGuard::install(Arc::clone(&observer));
    crate::ast::begin_parse_count(&root);
    let with_tests = crate::ast::with_request_parse_cache(|| {
        compute(&ImportersArgs {
            tests: true,
            ..base
        })
        .unwrap()
    });
    let parse_counts = crate::ast::finish_parse_count(&root);
    let work = observer.snapshot().work;
    assert_eq!(with_tests.direct_importers, ordinary.direct_importers);
    assert_eq!(with_tests.dependents_count, ordinary.dependents_count);
    let importer_impact = with_tests.test_impact.expect("test impact present");
    assert_eq!(importer_impact.tests, ["tests/owner.test.ts"]);
    let standalone = crate::tests::impact::generate_impact_plan(&crate::tests::ImpactArgs {
        entrypoints: vec![root.join("src/subject.ts").display().to_string()],
        entrypoint_symbols: vec![Some(String::new())],
        include_symbols: false,
        root: root.clone(),
        config: None,
        tsconfig: None,
        format: None,
        json: false,
    })
    .unwrap();
    assert_eq!(
        standalone
            .selected_tests
            .into_iter()
            .map(|test| test.test_file)
            .collect::<Vec<_>>(),
        importer_impact.tests
    );
    // Runner setup parsing records a strict diagnostic before the one union
    // fact collection recollects its recovered imports. The standard recovered
    // mode must reuse that AST rather than parse the helper a second time.
    assert_eq!(work["ts_facts.collections"], 1, "{work:#?}");
    assert_eq!(parse_counts.get(&root.join("vitest.config.ts")), Some(&1));
    assert_eq!(parse_counts.get(&root.join("setup.ts")), Some(&1));
}

#[test]
fn tests_flag_reuses_one_discovery_parse_and_graph_pipeline() {
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let _guard = crate::diagnostics::InvocationGuard::install(Arc::clone(&observer));

    compute(&args("util.ts", true)).unwrap();

    let work = observer.snapshot().work;
    assert_eq!(work["discovery.roots"], 1, "{work:#?}");
    assert_eq!(work["graph.builds"], 1, "{work:#?}");
    assert_eq!(work["symbol_index.builds"], 1, "{work:#?}");
    assert_eq!(work["ts_facts.files"], work["parse.files"], "{work:#?}");
    assert_eq!(work["parse.requests"], work["parse.files"], "{work:#?}");
}

#[test]
fn tests_flag_surfaces_strict_vitest_discovery_errors() {
    // A syntactically valid Vitest config with an invalid discovery glob is a
    // project error. `importers --tests` must preserve that graph-preparation
    // failure instead of returning a partial impact report.
    let root = test_plan_fixture_root("impact-invalid-vitest-discovery");
    let result = compute(&ImportersArgs {
        file: PathBuf::from("src/Service.cs"),
        tests: true,
        root: Some(root),
        tsconfig: None,
        format: None,
        json: false,
    });
    let error = match result {
        Err(error) => error,
        Ok(_) => panic!("invalid Vitest discovery must fail the impact graph"),
    };

    let detail = format!("{error:#}");
    assert!(detail.contains("error parsing glob"), "{detail}");
}

#[test]
fn renders_formats_and_runs() {
    let report = compute(&args("util.ts", true)).unwrap();
    let mut human = Vec::new();
    render(&report, Format::Human, &mut human).unwrap();
    let text = String::from_utf8(human).unwrap();
    assert!(text.contains("util.ts (3 dependents)"));
    assert!(text.contains("impacts 1 tests:"));
    assert!(text.contains("consumer.test.ts"));

    let mut paths = Vec::new();
    render(&report, Format::Paths, &mut paths).unwrap();
    assert!(String::from_utf8(paths).unwrap().contains("barrel.ts"));

    for format in [Format::Json, Format::Yml, Format::Md] {
        let mut buf = Vec::new();
        render(&report, format, &mut buf).unwrap();
        assert!(!buf.is_empty());
    }

    // Human render without --tests: the test-impact section is omitted.
    let no_tests = compute(&args("util.ts", false)).unwrap();
    let mut human = Vec::new();
    render(&no_tests, Format::Human, &mut human).unwrap();
    let text = String::from_utf8(human).unwrap();
    assert!(!text.contains("impacts"));

    let _ = run(args("util.ts", false)).unwrap();
}

#[test]
fn expired_deadline_rejects_output_before_rendering() {
    let report = compute(&args("util.ts", false)).unwrap();
    let _deadline = crate::invocation::install_test_deadline(std::time::Duration::ZERO).unwrap();
    let mut output = Vec::new();

    let error = render(&report, Format::Json, &mut output).unwrap_err();

    assert!(error.to_string().contains("timed out"));
    assert!(output.is_empty());
}
