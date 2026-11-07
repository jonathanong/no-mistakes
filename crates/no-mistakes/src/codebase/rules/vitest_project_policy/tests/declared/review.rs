use super::*;

fn run_fixture(case: &str, files: &[&str]) -> Diagnostics {
    let root = crate::codebase::ts_resolver::normalize_path(
        &crate::test_support::rule_fixture_root("vitest-project-policy").join(case),
    );
    let files = files.iter().map(|file| root.join(file)).collect::<Vec<_>>();
    let sources = SourceStore::new(std::sync::Arc::new(
        crate::codebase::ts_source::FileInventory::from_paths(&files),
    ));
    let config =
        crate::config::v2::load_v2_config(&root, Some(&root.join(".no-mistakes.yml"))).unwrap();
    diagnostics_with_files_and_sources(&root, &config, &files, &sources, false).unwrap()
}

#[test]
fn helper_filters_preserve_effective_resolution_and_source_suppressions() {
    let output = run_fixture(
        "declared-filter",
        &[
            "vitest.config.ts",
            "helpers/included.mts",
            "helpers/excluded.mts",
            "helpers/excluded-broken.mts",
            "helpers/omitted.mts",
            "helpers/omitted-broken.mts",
            "helpers/suppressed-file.mts",
            "helpers/suppressed-line.mts",
            "helpers/suppressed-next.mts",
        ],
    );
    assert!(output.incomplete.is_empty(), "{:?}", output.incomplete);
    assert_eq!(output.findings.len(), 1, "{:?}", output.findings);
    assert_eq!(output.findings[0].file, "helpers/included.mts");
    assert_eq!(output.findings[0].line, 1);
}

#[test]
fn glob_matched_source_failures_do_not_hide_other_findings() {
    let output = run_fixture(
        "declared-source-errors",
        &[
            "vitest.config.ts",
            "helpers/good.mts",
            "helpers/broken.mts",
            "helpers/suppressed.mts",
            "helpers/vanished.mts",
        ],
    );
    assert_eq!(output.findings.len(), 1);
    assert_eq!(output.findings[0].file, "helpers/good.mts");
    let files = output
        .incomplete
        .iter()
        .map(|failure| {
            failure
                .file
                .as_ref()
                .unwrap()
                .to_string_lossy()
                .into_owned()
        })
        .collect::<Vec<_>>();
    assert_eq!(
        files,
        vec!["helpers/broken.mts", "helpers/vanished.mts", "missing.ts"]
    );
    assert!(output
        .incomplete
        .iter()
        .all(|failure| failure.consumer == RULE_ID
            && failure.property.as_deref() == Some("declared")));
}

#[test]
fn findings_only_entrypoint_returns_source_failure_instead_of_success() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &crate::test_support::rule_fixture_root("vitest-project-policy")
            .join("declared-source-errors"),
    );
    let files = vec![
        root.join("vitest.config.ts"),
        root.join("helpers/broken.mts"),
    ];
    let config =
        crate::config::v2::load_v2_config(&root, Some(&root.join(".no-mistakes.yml"))).unwrap();
    let error = check_with_files(&root, &config, &files).unwrap_err();
    let failure = error
        .downcast_ref::<crate::codebase::check_outcome::CheckInputFailure>()
        .unwrap();
    assert_eq!(
        failure.file.as_deref(),
        Some(Path::new("helpers/broken.mts"))
    );
    assert_eq!(failure.property.as_deref(), Some("declared"));
    assert!(failure.message.contains("cannot parse configured source"));
}
