use super::*;

fn dynamic_import_fixture() -> std::path::PathBuf {
    fixture("codebase-analysis/test-no-unmocked-dynamic-imports")
}

fn dynamic_import_test_facts(
    path: &std::path::Path,
    source: &str,
) -> crate::codebase::check_facts::CheckFileFacts {
    crate::codebase::check_facts::CheckFileFacts {
        ts: crate::codebase::ts_source::facts::TsFileFacts {
            imports: crate::codebase::dependencies::extract::ImportExtractor::for_typescript()
                .unwrap()
                .extract(source)
                .unwrap(),
            ..Default::default()
        },
        source: Some(source.to_string()),
        dynamic_imports: Some(
            test_no_unmocked_dynamic_imports::ast::extract(path, source).unwrap(),
        ),
        ..Default::default()
    }
}

#[test]
fn run_check_with_facts_rejects_missing_prepared_graph_test_facts() {
    let root = dynamic_import_fixture();
    let test = root.join("tests/bad.test.mts");
    let shared = crate::codebase::check_facts::CheckFactMap {
        files: vec![test],
        graph_plan: crate::codebase::ts_source::facts::TsFactPlan::imports(),
        ..Default::default()
    };
    let error = run_check_with_facts(&root, None, None, &shared).unwrap_err();
    assert!(error
        .to_string()
        .contains("prepared graph facts are missing 1 indexable file"));
}

#[test]
fn run_check_with_facts_reports_missing_source_and_dynamic_facts() {
    let root = dynamic_import_fixture();
    let test = root.join("tests/bad.test.mts");
    let mut shared = crate::codebase::check_facts::CheckFactMap {
        files: vec![test.clone()],
        graph_plan: crate::codebase::ts_source::facts::TsFactPlan::imports(),
        ..Default::default()
    };
    shared.ts.insert(test.clone(), Default::default());
    assert!(format!(
        "{:#}",
        run_check_with_facts(&root, None, None, &shared).unwrap_err()
    )
    .contains("missing source facts"));
    shared.ts.insert(
        test,
        crate::codebase::check_facts::CheckFileFacts {
            source: Some("it('x', async () => {})".to_string()),
            ..Default::default()
        },
    );
    assert!(format!(
        "{:#}",
        run_check_with_facts(&root, None, None, &shared).unwrap_err()
    )
    .contains("missing dynamic import facts"));
}

#[test]
fn run_check_with_facts_skips_disabled_parse_errors() {
    let root = dynamic_import_fixture();
    let test = root.join("tests/disabled.test.mts");
    let mut shared = crate::codebase::check_facts::CheckFactMap {
        files: vec![test.clone()],
        graph_plan: crate::codebase::ts_source::facts::TsFactPlan::imports(),
        ..Default::default()
    };
    shared.ts.insert(test, crate::codebase::check_facts::CheckFileFacts {
        source: Some("// no-mistakes-disable-file test-no-unmocked-dynamic-imports\nexport const Broken =".to_string()),
        parse_error: Some("bad syntax".to_string()),
        ..Default::default()
    });
    run_check_with_facts(&root, None, None, &shared).unwrap();
}

#[test]
fn run_check_with_facts_executes_valid_shared_facts() {
    let root = dynamic_import_fixture();
    let facts = crate::codebase::check_facts::collect_check_facts(
        &root,
        crate::codebase::ts_source::discover_files(&root, &[]),
        crate::codebase::check_facts::CheckFactPlan {
            imports: true,
            dynamic_imports: true,
            source: true,
            ..Default::default()
        },
    );
    let aggregate = run_check_with_facts(&root, None, None, &facts).unwrap();
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let legacy =
        test_no_unmocked_dynamic_imports::check_with_facts(&root, &config, None, &facts).unwrap();
    assert_eq!(legacy, aggregate);
}

#[test]
fn run_check_with_facts_resolves_setup_mocks() {
    let root = dynamic_import_fixture();
    let test = root.join("tests/setup-good.test.mts");
    let setup = root.join("tests/setup-vitest.mts");
    let mut shared = crate::codebase::check_facts::CheckFactMap {
        files: vec![test.clone(), setup.clone()],
        graph_plan: crate::codebase::ts_source::facts::TsFactPlan::imports(),
        ..Default::default()
    };
    shared.ts.insert(test.clone(), dynamic_import_test_facts(&test,
        "import { expect, test } from 'vitest'\ntest('setup file mock counts', async () => { const mod = await import('@lib/setup-target.mts'); expect(mod.setupValue).toBe('mocked') })\n"));
    shared.ts.insert(setup.clone(), dynamic_import_test_facts(&setup,
        "import { vi } from 'vitest'\nvi.mock('@lib/setup-target.mts', () => ({ setupValue: 'mocked' }))\n"));
    run_check_with_facts(&root, None, None, &shared).unwrap();
}

#[test]
fn run_check_with_facts_skips_reachable_deps_with_parse_errors() {
    let root = dynamic_import_fixture();
    let test = root.join("tests/bad.test.mts");
    let files = vec![
        test.clone(),
        root.join("tests/setup-vitest.mts"),
        root.join("src/unreadable.mts"),
    ];
    let facts = crate::codebase::check_facts::collect_check_facts(
        &root,
        files,
        crate::codebase::check_facts::CheckFactPlan {
            imports: true,
            dynamic_imports: true,
            source: true,
            ..Default::default()
        },
    );
    let mut shared = crate::codebase::check_facts::CheckFactMap {
        files: facts.files().to_vec(),
        ts: facts.ts,
        graph_plan: facts.graph_plan,
        ..Default::default()
    };
    shared.ts.insert(test.clone(), dynamic_import_test_facts(&test,
        "import '@lib/unreadable.mts'\ntest('bad', async () => { await import('@lib/setup-target.mts') })\n"));
    run_check_with_facts(&root, None, None, &shared).unwrap();
}

#[test]
fn run_check_with_facts_uses_shared_graph_without_reachable_dep_disk_fallback() {
    let root = dynamic_import_fixture();
    let test = root.join("tests/bad.test.mts");
    let setup = root.join("tests/setup-vitest.mts");
    let mut shared = crate::codebase::check_facts::CheckFactMap {
        files: vec![test.clone(), setup.clone()],
        graph_plan: crate::codebase::ts_source::facts::TsFactPlan::imports(),
        ..Default::default()
    };
    shared.ts.insert(test.clone(), dynamic_import_test_facts(&test,
        "import '@lib/unreadable.mts'\ntest('bad', async () => { await import('@lib/setup-target.mts') })\n"));
    let setup_source = std::fs::read_to_string(&setup).unwrap();
    shared.ts.insert(
        setup.clone(),
        dynamic_import_test_facts(&setup, &setup_source),
    );
    assert!(run_check_with_facts(&root, None, None, &shared)
        .unwrap()
        .is_empty());
}

#[test]
fn run_check_with_facts_reports_missing_setup_fact_shapes() {
    let root = dynamic_import_fixture();
    let test = root.join("tests/setup-good.test.mts");
    let setup = root.join("tests/setup-vitest.mts");
    let mut shared = crate::codebase::check_facts::CheckFactMap {
        files: vec![test.clone()],
        graph_files: vec![test.clone(), setup.clone()],
        graph_files_complete: true,
        graph_plan: crate::codebase::ts_source::facts::TsFactPlan::imports(),
        ..Default::default()
    };
    shared.ts.insert(test.clone(), dynamic_import_test_facts(&test,
        "test('setup file mock counts', async () => { await import('@lib/setup-target.mts') })\n"));
    assert!(run_check_with_facts(&root, None, None, &shared)
        .unwrap_err()
        .to_string()
        .contains("prepared graph facts are missing 1 indexable file"));
    shared.ts.insert(
        setup.clone(),
        crate::codebase::check_facts::CheckFileFacts {
            parse_error: Some("bad setup".to_string()),
            ..Default::default()
        },
    );
    assert!(run_check_with_facts(&root, None, None, &shared)
        .unwrap_err()
        .to_string()
        .contains("bad setup"));
    shared.ts.insert(
        setup,
        crate::codebase::check_facts::CheckFileFacts {
            source: Some("vi.mock('@lib/setup-target.mts')".to_string()),
            ..Default::default()
        },
    );
    assert!(run_check_with_facts(&root, None, None, &shared)
        .unwrap_err()
        .to_string()
        .contains("missing dynamic import facts"));
}

#[test]
fn run_check_with_facts_reports_test_file_parse_error() {
    let root = dynamic_import_fixture();
    let test = root.join("tests/bad.test.mts");
    let mut shared = crate::codebase::check_facts::CheckFactMap {
        files: vec![test.clone()],
        graph_plan: crate::codebase::ts_source::facts::TsFactPlan::imports(),
        ..Default::default()
    };
    shared.ts.insert(
        test,
        crate::codebase::check_facts::CheckFileFacts {
            source: Some("test('broken', () => {})".to_string()),
            parse_error: Some("syntax error".to_string()),
            ..Default::default()
        },
    );
    assert!(format!(
        "{:#}",
        run_check_with_facts(&root, None, None, &shared).unwrap_err()
    )
    .contains("syntax error"));
}

#[test]
fn filesystem_rule_ids_are_distinct() {
    let ids = [
        AGENTS_MD_MAX_SIZE,
        RUST_MAX_LINES_PER_FILE,
        RUST_NO_INLINE_TESTS,
        RUST_NO_INLINE_ALLOWS,
        NEXTJS_NO_API_ROUTES,
        NEXTJS_NO_CACHING,
    ];
    for (index, id) in ids.iter().enumerate() {
        assert!(!ids[index + 1..].contains(id));
    }
}

#[test]
fn run_filesystem_rules_returns_empty_when_not_configured() {
    assert!(run_filesystem_rules(std::path::Path::new("/tmp"), None)
        .unwrap()
        .is_empty());
}

#[test]
fn run_filesystem_rules_execute_enabled_rules() {
    for (fixture_path, rule) in [
        (
            "codebase-analysis/filesystem-rules/agents-md-max-size",
            AGENTS_MD_MAX_SIZE,
        ),
        (
            "codebase-analysis/filesystem-rules/rust-max-lines-per-file",
            RUST_MAX_LINES_PER_FILE,
        ),
        (
            "codebase-analysis/filesystem-rules/rust-no-inline-tests",
            RUST_NO_INLINE_TESTS,
        ),
        ("rules/rust-no-inline-allows/fail", RUST_NO_INLINE_ALLOWS),
    ] {
        let root = fixture(fixture_path);
        let findings = run_filesystem_rules(&root, Some(&root.join(".no-mistakes.yml"))).unwrap();
        assert!(findings.iter().any(|finding| finding.rule == rule));
    }
}

#[test]
fn run_filesystem_rules_applies_shared_suppression() {
    let root = fixture("rules/banned-renamed-files/fail");
    let findings = run_filesystem_rules(&root, Some(&root.join(".no-mistakes.yml"))).unwrap();
    assert_eq!(
        findings
            .iter()
            .map(|finding| finding.file.as_str())
            .collect::<Vec<_>>(),
        vec!["web/middleware.ts"]
    );
}

#[test]
fn run_filesystem_rules_with_files_executes_all_enabled_rust_rules() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/check-runner/facts-and-filesystem/fixture");
    let findings = run_filesystem_rules_with_files(
        &root,
        Some(&root.join(".no-mistakes.yml")),
        &[root.join("src/lib.rs")],
    )
    .unwrap();
    assert!(findings.iter().any(|f| f.rule == RUST_MAX_LINES_PER_FILE));
    assert!(findings.iter().any(|f| f.rule == RUST_NO_INLINE_TESTS));
}

#[test]
fn run_check_with_facts_surfaces_invalid_tsconfig() {
    let root = dynamic_import_fixture();
    let error = run_check_with_facts(
        &root,
        None,
        Some(&root.join("nonexistent-tsconfig.json")),
        &crate::codebase::check_facts::CheckFactMap::default(),
    )
    .unwrap_err();
    assert!(format!("{error:#}").contains("nonexistent-tsconfig.json"));
}

#[test]
fn run_check_with_facts_returns_empty_when_no_codebase_rules_enabled() {
    let tmp = tempfile::tempdir().unwrap();
    let findings = run_check_with_facts(
        tmp.path(),
        None,
        None,
        &crate::codebase::check_facts::CheckFactMap::default(),
    )
    .unwrap();
    assert!(findings.is_empty());
}
