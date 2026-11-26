use super::super::*;

#[test]
fn forbidden_paths_options_validate_emptiness_duplicates_overlap_and_messages() {
    let empty_entry = options::Options {
        forbidden_paths: vec!["./".into()],
        ..options::Options::default()
    };
    let error = options::validate(&empty_entry).unwrap_err().to_string();
    assert!(error.contains("Expected a non-empty path"), "{error}");

    let duplicate = options::Options {
        forbidden_paths: vec![
            "~/.local/share/example-store".into(),
            "~/.local/share/example-store/".into(),
        ],
        ..options::Options::default()
    };
    let error = options::validate(&duplicate).unwrap_err().to_string();
    assert!(
        error.contains("Expected a unique forbidden path"),
        "{error}"
    );

    let overlap = options::Options {
        caches: vec![super::spec(
            ".cache/widgets",
            "widgets.lock",
            &["runner.os"],
        )],
        forbidden_paths: vec!["./.cache/widgets/".into()],
        ..options::Options::default()
    };
    let error = options::validate(&overlap).unwrap_err().to_string();
    assert!(
        error.contains("Expected a path absent from options.caches[].path"),
        "{error}"
    );

    let blank_message = options::Options {
        forbidden_paths: vec!["~/.local/share/example-store".into()],
        forbidden_paths_message: Some("   ".into()),
        ..options::Options::default()
    };
    let error = options::validate(&blank_message).unwrap_err().to_string();
    assert!(error.contains("Expected a non-empty message"), "{error}");

    let forbidden_only = options::Options {
        forbidden_paths: vec!["~/.local/share/example-store".into()],
        forbidden_paths_message: Some("Do not cache this path.".into()),
        ..options::Options::default()
    };
    assert!(options::validate(&forbidden_only).is_ok());
}

#[test]
fn forbidden_paths_reports_workflows_and_composite_actions_and_respects_messages() {
    let default_findings = super::rows("forbidden-paths", "forbidden-paths");
    assert_eq!(default_findings.len(), 8, "{default_findings:?}");
    super::assert_needles(
        &default_findings,
        "cache.yml",
        &[
            "Caching `~/.local/share/example-store` is forbidden",
            "Caching `~/.local/share/example-store` is forbidden",
            "Caching `.cache/example-store` is forbidden",
            "Caching `~/.local/share/example-store` is forbidden",
        ],
    );
    super::assert_needles(
        &default_findings,
        "/store-cache/action.yml",
        &["Caching `~/.local/share/example-store` is forbidden"],
    );
    for file in [
        "unaligned-forbidden.yml",
        "aliased-path.yml",
        "aliased-store-cache/action.yml",
    ] {
        super::assert_needles(&default_findings, file, &["could not be aligned"]);
    }
    super::assert_silent(&default_findings, "aliased-other-path.yml");
    super::assert_silent(&default_findings, "node-store/action.yml");

    let custom_findings = super::rows("forbidden-paths", "forbidden-paths-custom-message");
    assert_eq!(custom_findings.len(), 8, "{custom_findings:?}");
    assert_eq!(
        custom_findings
            .iter()
            .filter(|finding| {
                finding.message == "Do not cache the example store; delete this cache step."
            })
            .count(),
        5,
        "{custom_findings:?}"
    );

    let rule_msg_findings = super::rows("forbidden-paths", "forbidden-paths-rule-message");
    assert_eq!(rule_msg_findings.len(), 8, "{rule_msg_findings:?}");
    assert_eq!(
        rule_msg_findings
            .iter()
            .filter(|finding| {
                finding.message == "Store caching is disabled in this repository; delete this step."
            })
            .count(),
        5,
        "{rule_msg_findings:?}"
    );

    let unset_findings = super::rows("forbidden-paths", "policy");
    assert!(unset_findings.is_empty(), "{unset_findings:?}");
}

#[test]
fn forbidden_message_option_overrides_rule_message_without_clobbering_cache_findings() {
    let workflow = "\
name: Mixed
jobs:
  unit:
    steps:
      - uses: actions/cache@v4
        with:
          path: |
            .cache/widgets
            ~/.local/share/example-store
          key: widgets-shared
";
    let jobs = super::expect_jobs(workflow, false);
    let options = options::Options {
        caches: vec![super::spec(
            ".cache/widgets",
            "widgets.lock",
            &["runner.os"],
        )],
        forbidden_paths: vec!["~/.local/share/example-store".into()],
        forbidden_paths_message: Some("Forbidden store path.".into()),
        ..options::Options::default()
    };
    let findings = super::judge_findings(
        ".github/workflows/mixed.yml",
        &jobs,
        &options,
        Some("Rule-level fallback."),
        &|_| None,
    );
    assert_eq!(findings.len(), 3, "{findings:?}");
    assert_eq!(
        findings
            .iter()
            .filter(|finding| finding.message == "Forbidden store path.")
            .count(),
        1,
        "{findings:?}"
    );
    assert_eq!(
        findings
            .iter()
            .filter(|finding| finding.message == "Rule-level fallback.")
            .count(),
        2,
        "{findings:?}"
    );
}

#[test]
fn two_forbidden_paths_in_one_step_report_each_path_on_the_path_line() {
    let workflow = "\
name: Multi Forbidden
jobs:
  unit:
    steps:
      - uses: actions/cache@v4
        with:
          path: |
            ~/.local/share/example-store
            .cache/example-store
          key: store-${{ runner.os }}
";
    let jobs = super::expect_jobs(workflow, false);
    let options = options::Options {
        forbidden_paths: vec![
            "~/.local/share/example-store".into(),
            ".cache/example-store".into(),
        ],
        ..options::Options::default()
    };
    let findings = super::judge_findings(
        ".github/workflows/multi.yml",
        &jobs,
        &options,
        None,
        &|_| None,
    );
    assert_eq!(findings.len(), 2, "{findings:?}");
    assert_eq!(findings[0].line, 7);
    assert_eq!(findings[1].line, 7);
    assert!(findings[0]
        .message
        .contains("`~/.local/share/example-store`"));
    assert!(findings[1].message.contains("`.cache/example-store`"));
}
