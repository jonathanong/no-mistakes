use super::*;
use std::path::PathBuf;

fn fixture(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/integration-tests")
            .join(name)
            .join("fixture"),
    )
}

fn fixture_file(file: &str) -> PathBuf {
    fixture("parse-errors").join(file)
}

#[test]
fn analyzer_reports_parse_context() {
    let file = fixture_file("src/syntax-error.ts");
    let err = test_support::analyze_files(&[file])
        .err()
        .expect("expected syntax error");
    assert!(err
        .to_string()
        .contains("analyzing integration annotations"));
}

#[test]
fn config_parsers_report_syntax_errors() {
    let root = fixture("parse-errors");

    let pw_path = root.join("playwright.syntax-error.ts");
    let pw_source = std::fs::read_to_string(&pw_path).unwrap();
    let tsconfig = test_support::tsconfig_without_config(&root);
    assert!(test_support::parse_playwright(&pw_source, &pw_path, &root, &tsconfig).is_err());

    let vitest_path = root.join("vitest.syntax-error.mts");
    let vitest_source = std::fs::read_to_string(&vitest_path).unwrap();
    let tsconfig = test_support::tsconfig_without_config(&root);
    assert!(
        test_support::parse_vitest(&vitest_source, &vitest_path, &root, &root, &tsconfig).is_err()
    );

    let root = fixture("coverage");
    let empty_path = root.join("vitest.empty-array-invalid.mts");
    let empty_source = std::fs::read_to_string(&empty_path).unwrap();
    let tsconfig = test_support::tsconfig_without_config(&root);
    assert!(
        test_support::parse_vitest(&empty_source, &empty_path, &root, &root, &tsconfig).is_err()
    );
}

#[test]
fn check_with_facts_reports_dropped_helper_parse_errors() {
    let root = fixture("basic");
    let file = root.join("helpers/openai.mts");
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(&root);
    let visible = snapshot.paths_for(&root);
    let config = crate::config::v2::load_v2_config_from_visible(&root, None, &visible).unwrap();
    let tsconfig =
        crate::codebase::ts_resolver::resolve_tsconfig_from_visible(None, &root, &visible).unwrap();
    let runner_configs = runner_config::prepare(&root, &config, &visible, &tsconfig);
    let files = crate::codebase::ts_source::discover_files_from_visible(&root, &[], &visible);
    let mut shared = crate::codebase::check_facts::collect_check_facts(
        &root,
        files,
        crate::codebase::check_facts::CheckFactPlan {
            integration: true,
            integration_runner_configs: Some(std::sync::Arc::new(runner_configs)),
            ..Default::default()
        },
    );
    shared.ts.insert(
        file,
        crate::codebase::check_facts::CheckFileFacts {
            parse_error: Some("synthetic helper parse error".to_string()),
            ..Default::default()
        },
    );

    let error =
        check_with_prepared_facts(&root, &config, &shared, &tsconfig, &snapshot).unwrap_err();

    assert!(error.to_string().contains("synthetic helper parse error"));
}
