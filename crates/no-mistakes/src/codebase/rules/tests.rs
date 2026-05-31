use super::*;
use crate::config::v2::schema::{RuleDef, RuleScope};

fn fixture(path: &str) -> std::path::PathBuf {
    let mut parts = path.splitn(3, '/');
    let category = parts.next().unwrap_or(path);
    let sub = parts.next().unwrap_or("");
    let rest = parts.next().unwrap_or("");
    let mut p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases")
        .join(category)
        .join(sub)
        .join("fixture");
    if !rest.is_empty() {
        p = p.join(rest);
    }
    crate::codebase::ts_resolver::normalize_path(&p)
}

#[test]
fn rule_enabled_requires_configured_rule() {
    let mut config = crate::config::v2::NoMistakesConfig::default();
    assert!(!rule_enabled(&config, TEST_NO_UNMOCKED_DYNAMIC_IMPORTS));
    config.rules.push(RuleDef {
        rule: TEST_NO_UNMOCKED_DYNAMIC_IMPORTS.to_string(),
        scope: Some(RuleScope::Repository),
        ..Default::default()
    });
    assert!(rule_enabled(&config, TEST_NO_UNMOCKED_DYNAMIC_IMPORTS));
}

#[test]
fn rule_enabled_accepts_project_rule_without_top_level_options() {
    let mut config = crate::config::v2::NoMistakesConfig::default();
    config.projects.insert(
        "tests".to_string(),
        crate::config::v2::schema::Project::default(),
    );
    config.rules.push(RuleDef {
        rule: TEST_NO_UNMOCKED_DYNAMIC_IMPORTS.to_string(),
        projects: vec!["tests".to_string()],
        ..Default::default()
    });
    assert!(rule_enabled(&config, TEST_NO_UNMOCKED_DYNAMIC_IMPORTS));
}

#[test]
fn run_check_returns_empty_when_rule_is_not_enabled() {
    let root = std::path::Path::new("/tmp/no-mistakes-empty-rules");
    let findings = run_check(root, None, None).unwrap();
    assert!(findings.is_empty());
}

#[test]
fn run_check_executes_enabled_rule() {
    let root = fixture("codebase-analysis/test-no-unmocked-dynamic-imports");

    run_check(&root, None, None).unwrap();
}

#[test]
fn run_check_executes_storybook_rule() {
    let root = fixture("rules/require-storybook-stories/covered");

    let findings = run_check(&root, None, None).unwrap();

    assert!(findings.is_empty(), "{findings:#?}");
}

#[test]
fn run_check_executes_playwright_rules() {
    let root = fixture("check-runner/playwright-unique-test-ids");

    let findings = run_check(&root, None, None).unwrap();

    assert!(findings
        .iter()
        .any(|finding| finding.rule == crate::playwright::rules::PLAYWRIGHT_UNIQUE_TEST_IDS));
}

#[test]
fn run_check_with_facts_executes_playwright_rules() {
    let root = fixture("check-runner/playwright-unique-test-ids");
    let facts = crate::codebase::check_facts::CheckFactMap::default();

    let findings = run_check_with_facts(&root, None, None, &facts).unwrap();

    assert!(findings
        .iter()
        .any(|finding| finding.rule == crate::playwright::rules::PLAYWRIGHT_UNIQUE_TEST_IDS));
}

#[test]
fn run_check_with_facts_propagates_playwright_rule_errors() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let config_path = root.join(".no-mistakes.yml");
    std::fs::write(
        &config_path,
        "tests:\n  playwright:\n    configs: missing.config.ts\nrules:\n  - rule: playwright-unique-test-ids\n    scope: repository\n",
    )
    .unwrap();
    let facts = crate::codebase::check_facts::CheckFactMap::default();

    let error = run_check_with_facts(root, Some(&config_path), None, &facts).unwrap_err();

    assert!(error
        .to_string()
        .contains("Playwright config does not exist"));
}

#[test]
fn run_check_with_facts_executes_storybook_rule() {
    let root = fixture("rules/require-storybook-stories/covered");
    let files = crate::codebase::ts_source::discover_files(&root, &[]);
    let facts = crate::codebase::check_facts::collect_check_facts(
        &root,
        files,
        crate::codebase::check_facts::CheckFactPlan {
            symbols: true,
            react: true,
            storybook: true,
            source: true,
            dynamic_imports: true,
            ..Default::default()
        },
    );

    let findings = run_check_with_facts(&root, None, None, &facts).unwrap();

    assert!(findings.is_empty(), "{findings:#?}");
}

#[test]
fn run_check_executes_forbidden_dependencies_rule() {
    let root = fixture("codebase-analysis/forbidden-dependencies-basic");
    let findings = run_check(&root, None, None).unwrap();
    assert!(findings.iter().any(|f| f.rule == FORBIDDEN_DEPENDENCIES));
}

#[test]
fn run_check_with_facts_executes_forbidden_dependencies_rule() {
    let root = fixture("codebase-analysis/forbidden-dependencies-basic");
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let graph_plan = crate::codebase::rules::forbidden_dependencies::graph_plan(&config)
        .expect("forbidden dependencies config requests graph facts");
    let (graph, graph_context) =
        crate::codebase::dependencies::graph::ts_fact_plan_and_context_for_plan(&root, graph_plan);
    let shared = crate::codebase::check_facts::collect_check_facts(
        &root,
        crate::codebase::ts_source::discover_files(&root, &[]),
        crate::codebase::check_facts::CheckFactPlan {
            graph,
            graph_context,
            ..Default::default()
        },
    );
    let findings = run_check_with_facts(&root, None, None, &shared).unwrap();
    assert!(findings.iter().any(|f| f.rule == FORBIDDEN_DEPENDENCIES));
}

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
fn run_check_with_facts_reports_missing_test_facts() {
    let root = dynamic_import_fixture();
    let test = root.join("tests/bad.test.mts");
    let shared = crate::codebase::check_facts::CheckFactMap {
        files: vec![test.clone()],
        ..Default::default()
    };

    let error = run_check_with_facts(&root, None, None, &shared).unwrap_err();

    assert!(error.to_string().contains("missing shared facts"));
}

#[test]
fn run_check_with_facts_reports_missing_source_and_dynamic_facts() {
    let root = dynamic_import_fixture();
    let test = root.join("tests/bad.test.mts");
    let mut shared = crate::codebase::check_facts::CheckFactMap {
        files: vec![test.clone()],
        ..Default::default()
    };
    shared.ts.insert(test.clone(), Default::default());

    let missing_source = run_check_with_facts(&root, None, None, &shared).unwrap_err();
    assert!(format!("{missing_source:#}").contains("missing source facts"));

    shared.ts.insert(
        test,
        crate::codebase::check_facts::CheckFileFacts {
            source: Some("it('x', async () => {})".to_string()),
            ..Default::default()
        },
    );
    let missing_dynamic = run_check_with_facts(&root, None, None, &shared).unwrap_err();
    assert!(format!("{missing_dynamic:#}").contains("missing dynamic import facts"));
}

#[test]
fn run_check_with_facts_skips_disabled_parse_errors() {
    let root = dynamic_import_fixture();
    let test = root.join("tests/disabled.test.mts");
    let source =
        "// no-mistakes-disable-file test-no-unmocked-dynamic-imports\nexport const Broken =";
    let mut shared = crate::codebase::check_facts::CheckFactMap {
        files: vec![test.clone()],
        ..Default::default()
    };
    shared.ts.insert(
        test,
        crate::codebase::check_facts::CheckFileFacts {
            source: Some(source.to_string()),
            parse_error: Some("bad syntax".to_string()),
            ..Default::default()
        },
    );

    run_check_with_facts(&root, None, None, &shared).unwrap();
}

#[test]
fn run_check_with_facts_executes_valid_shared_facts() {
    let root = dynamic_import_fixture();
    let files = crate::codebase::ts_source::discover_files(&root, &[]);
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

    run_check_with_facts(&root, None, None, &facts).unwrap();
}

#[test]
fn run_check_with_facts_resolves_setup_mocks() {
    let root = dynamic_import_fixture();
    let test = root.join("tests/setup-good.test.mts");
    let setup = root.join("tests/setup-vitest.mts");
    let mut shared = crate::codebase::check_facts::CheckFactMap {
        files: vec![test.clone(), setup.clone()],
        ..Default::default()
    };
    shared.ts.insert(
        test.clone(),
        dynamic_import_test_facts(
            &test,
            "import { expect, test } from 'vitest'\n\
test('setup file mock counts', async () => {\n\
  const mod = await import('@lib/setup-target.mts')\n\
  expect(mod.setupValue).toBe('mocked')\n\
})\n",
        ),
    );
    shared.ts.insert(
        setup.clone(),
        dynamic_import_test_facts(
            &setup,
            "import { vi } from 'vitest'\n\
vi.mock('@lib/setup-target.mts', () => ({ setupValue: 'mocked' }))\n",
        ),
    );

    run_check_with_facts(&root, None, None, &shared).unwrap();
}

#[test]
fn run_check_with_facts_skips_reachable_deps_with_parse_errors() {
    let root = dynamic_import_fixture();
    let test = root.join("tests/bad.test.mts");
    let setup = root.join("tests/setup-vitest.mts");
    // src/unreadable.mts is a directory on disk, so collect_check_facts will
    // store a parse_error for it in CheckFactMap.
    let unreadable = root.join("src/unreadable.mts");
    let files = vec![test.clone(), setup, unreadable];
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
    let source = "import '@lib/unreadable.mts'\n\
test('bad', async () => {\n\
  await import('@lib/setup-target.mts')\n\
})\n";
    let files = facts.files().to_vec();
    let mut shared = crate::codebase::check_facts::CheckFactMap {
        files,
        ts: facts.ts,
        ..Default::default()
    };
    shared
        .ts
        .insert(test.clone(), dynamic_import_test_facts(&test, source));

    // Reachable deps with parse_error in CheckFactMap are silently skipped
    // rather than re-attempted from disk, so the check succeeds.
    run_check_with_facts(&root, None, None, &shared).unwrap();
}

#[test]
fn run_check_with_facts_uses_shared_graph_without_reachable_dep_disk_fallback() {
    // unreadable.mts is a directory. Shared-fact graph construction must not
    // fall back to reading it from disk when dependency facts are already
    // supplied for the importing test/setup files.
    let root = dynamic_import_fixture();
    let test = root.join("tests/bad.test.mts");
    let setup = root.join("tests/setup-vitest.mts");
    let source = "import '@lib/unreadable.mts'\n\
test('bad', async () => {\n\
  await import('@lib/setup-target.mts')\n\
})\n";
    let setup_source = std::fs::read_to_string(&setup).unwrap();
    let mut shared = crate::codebase::check_facts::CheckFactMap {
        files: vec![test.clone(), setup.clone()],
        ..Default::default()
    };
    shared
        .ts
        .insert(test.clone(), dynamic_import_test_facts(&test, source));
    shared.ts.insert(
        setup.clone(),
        dynamic_import_test_facts(&setup, &setup_source),
    );
    let findings = run_check_with_facts(&root, None, None, &shared).unwrap();
    assert!(findings.is_empty());
}

#[test]
fn run_check_with_facts_reports_missing_setup_fact_shapes() {
    let root = dynamic_import_fixture();
    let test = root.join("tests/setup-good.test.mts");
    let setup = root.join("tests/setup-vitest.mts");
    let test_source = "test('setup file mock counts', async () => {\n\
  await import('@lib/setup-target.mts')\n\
})\n";
    let mut shared = crate::codebase::check_facts::CheckFactMap {
        files: vec![test.clone()],
        ..Default::default()
    };
    shared
        .ts
        .insert(test.clone(), dynamic_import_test_facts(&test, test_source));

    let missing = run_check_with_facts(&root, None, None, &shared).unwrap_err();
    assert!(missing.to_string().contains("missing shared facts"));

    shared.files.push(setup.clone());
    shared.ts.insert(
        setup.clone(),
        crate::codebase::check_facts::CheckFileFacts {
            parse_error: Some("bad setup".to_string()),
            ..Default::default()
        },
    );
    let parse_error = run_check_with_facts(&root, None, None, &shared).unwrap_err();
    assert!(parse_error.to_string().contains("bad setup"));

    shared.ts.insert(
        setup,
        crate::codebase::check_facts::CheckFileFacts {
            source: Some("vi.mock('@lib/setup-target.mts')".to_string()),
            ..Default::default()
        },
    );
    let missing_dynamic = run_check_with_facts(&root, None, None, &shared).unwrap_err();
    assert!(missing_dynamic
        .to_string()
        .contains("missing dynamic import facts"));
}

#[test]
fn run_check_with_facts_reports_test_file_parse_error() {
    // with_facts.rs:48 — parse_error bail for the test file itself (without disable comment)
    let root = dynamic_import_fixture();
    let test = root.join("tests/bad.test.mts");
    let mut shared = crate::codebase::check_facts::CheckFactMap {
        files: vec![test.clone()],
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
    let error = run_check_with_facts(&root, None, None, &shared).unwrap_err();
    assert!(format!("{error:#}").contains("syntax error"));
}

#[test]
fn filesystem_rule_ids_are_distinct() {
    assert_ne!(AGENTS_MD_MAX_SIZE, RUST_MAX_LINES_PER_FILE);
    assert_ne!(RUST_MAX_LINES_PER_FILE, RUST_NO_INLINE_TESTS);
    assert_ne!(RUST_MAX_LINES_PER_FILE, RUST_NO_INLINE_ALLOWS);
    assert_ne!(RUST_MAX_LINES_PER_FILE, NEXTJS_NO_API_ROUTES);
    assert_ne!(RUST_MAX_LINES_PER_FILE, NEXTJS_NO_CACHING);
    assert_ne!(AGENTS_MD_MAX_SIZE, RUST_NO_INLINE_TESTS);
    assert_ne!(AGENTS_MD_MAX_SIZE, RUST_NO_INLINE_ALLOWS);
    assert_ne!(AGENTS_MD_MAX_SIZE, NEXTJS_NO_API_ROUTES);
    assert_ne!(AGENTS_MD_MAX_SIZE, NEXTJS_NO_CACHING);
    assert_ne!(RUST_NO_INLINE_TESTS, RUST_NO_INLINE_ALLOWS);
    assert_ne!(RUST_NO_INLINE_TESTS, NEXTJS_NO_API_ROUTES);
    assert_ne!(RUST_NO_INLINE_TESTS, NEXTJS_NO_CACHING);
    assert_ne!(RUST_NO_INLINE_ALLOWS, NEXTJS_NO_API_ROUTES);
    assert_ne!(RUST_NO_INLINE_ALLOWS, NEXTJS_NO_CACHING);
    assert_ne!(NEXTJS_NO_API_ROUTES, NEXTJS_NO_CACHING);
}

#[test]
fn run_filesystem_rules_returns_empty_when_not_configured() {
    let root = std::path::Path::new("/tmp");
    let findings = run_filesystem_rules(root, None).unwrap();
    assert!(findings.is_empty());
}

#[test]
fn run_filesystem_rules_executes_enabled_agents_md_rule() {
    let root = fixture("codebase-analysis/filesystem-rules/agents-md-max-size");
    let config = root.join(".no-mistakes.yml");
    let findings = run_filesystem_rules(&root, Some(&config)).unwrap();
    assert!(!findings.is_empty());
    assert!(findings.iter().any(|f| f.rule == AGENTS_MD_MAX_SIZE));
}

#[test]
fn run_filesystem_rules_executes_enabled_rust_max_lines_rule() {
    let root = fixture("codebase-analysis/filesystem-rules/rust-max-lines-per-file");
    let config = root.join(".no-mistakes.yml");
    let findings = run_filesystem_rules(&root, Some(&config)).unwrap();
    assert!(!findings.is_empty());
    assert!(findings.iter().any(|f| f.rule == RUST_MAX_LINES_PER_FILE));
}

#[test]
fn run_filesystem_rules_executes_enabled_rust_no_inline_tests_rule() {
    let root = fixture("codebase-analysis/filesystem-rules/rust-no-inline-tests");
    let config = root.join(".no-mistakes.yml");
    let findings = run_filesystem_rules(&root, Some(&config)).unwrap();
    assert!(!findings.is_empty());
    assert!(findings.iter().any(|f| f.rule == RUST_NO_INLINE_TESTS));
}

#[test]
fn run_filesystem_rules_executes_enabled_rust_no_inline_allows_rule() {
    let root = fixture("rules/rust-no-inline-allows/fail");
    let config = root.join(".no-mistakes.yml");
    let findings = run_filesystem_rules(&root, Some(&config)).unwrap();
    assert!(!findings.is_empty());
    assert!(findings.iter().any(|f| f.rule == RUST_NO_INLINE_ALLOWS));
}

#[test]
fn run_filesystem_rules_applies_shared_suppression() {
    let root = fixture("rules/banned-renamed-files/fail");
    let config = root.join(".no-mistakes.yml");

    let findings = run_filesystem_rules(&root, Some(&config)).unwrap();
    let files: Vec<_> = findings
        .iter()
        .map(|finding| finding.file.as_str())
        .collect();

    assert_eq!(files, vec!["web/middleware.ts"]);
}

#[test]
fn run_filesystem_rules_with_files_executes_all_enabled_rust_rules() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/check-runner/facts-and-filesystem/fixture");
    let config = root.join(".no-mistakes.yml");
    let files = vec![root.join("src/lib.rs")];

    let findings = run_filesystem_rules_with_files(&root, Some(&config), &files).unwrap();

    assert!(findings.iter().any(|f| f.rule == RUST_MAX_LINES_PER_FILE));
    assert!(findings.iter().any(|f| f.rule == RUST_NO_INLINE_TESTS));
}

#[test]
fn run_check_with_facts_surfaces_invalid_tsconfig() {
    let root = dynamic_import_fixture();
    let invalid_tsconfig = root.join("nonexistent-tsconfig.json");
    let shared = crate::codebase::check_facts::CheckFactMap::default();

    let error = run_check_with_facts(&root, None, Some(&invalid_tsconfig), &shared).unwrap_err();

    assert!(
        format!("{error:#}").contains("nonexistent-tsconfig.json"),
        "expected tsconfig path in error, got: {error:#}"
    );
}

#[test]
fn run_check_with_facts_returns_empty_when_no_codebase_rules_enabled() {
    // Exercises run.rs line 48: early return when any_codebase_rule_enabled is false.
    let tmp = tempfile::tempdir().unwrap();
    // Default config has no rules configured → any_codebase_rule_enabled returns false.
    let shared = crate::codebase::check_facts::CheckFactMap::default();
    let findings = run_check_with_facts(tmp.path(), None, None, &shared).unwrap();
    assert!(
        findings.is_empty(),
        "expected empty findings when no codebase rules are enabled: {findings:?}"
    );
}
