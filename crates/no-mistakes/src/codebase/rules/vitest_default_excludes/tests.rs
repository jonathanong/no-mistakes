use super::*;

fn fixture_root() -> PathBuf {
    normalize_path(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/rules/vitest-default-excludes"),
    )
}

fn load(root: &Path, config: &str) -> NoMistakesConfig {
    crate::config::v2::load_v2_config(root, Some(&root.join(config))).unwrap()
}

fn run(config: &str) -> Vec<RuleFinding> {
    let root = fixture_root();
    let config = load(&root, config);
    let files = [
        "vitest.config.ts",
        "vitest.external.ts",
        "custom.ts",
        "absent.ts",
        "suppressed.ts",
        "shadowed.ts",
    ]
    .map(|path| root.join(path));
    let findings =
        super::super::filesystem_dispatch::run_filesystem_rules_with_config(&root, &config, &files)
            .unwrap();
    assert_eq!(check_with_files(&root, &config, &files).unwrap(), findings);
    findings
}

#[test]
fn reports_only_literal_test_exclusion_overrides() {
    let findings = run(".no-mistakes.yml");
    assert_eq!(
        findings
            .iter()
            .map(|finding| (finding.file.as_str(), finding.line))
            .collect::<Vec<_>>(),
        [
            ("vitest.config.ts", 10),
            ("vitest.config.ts", 14),
            ("vitest.config.ts", 27),
            ("vitest.config.ts", 34),
        ]
    );
    assert!(findings[0].message.contains("**/.git/**"));
    assert!(!findings[0].message.contains("**/node_modules/**"));
    assert!(findings[1]
        .message
        .contains("**/node_modules/**, **/.git/**"));
    assert_eq!(findings[3].message, findings[0].message);
}

#[test]
fn configured_globs_replace_defaults_and_allow_message_override() {
    let findings = run("custom.yml");
    assert_eq!(findings.len(), 2);
    assert_eq!([findings[0].line, findings[1].line], [5, 8]);
    assert!(findings
        .iter()
        .all(|finding| finding.message == "Keep the generated tests excluded."));
}

#[test]
fn a_bad_config_retains_other_config_findings_and_records_its_failure() {
    let root = fixture_root();
    let config = load(&root, "mixed.yml");
    let session = AnalysisSession::disabled();
    session.enable_partial_checks();
    let files = [root.join("vitest.external.ts")];
    let sources = super::super::source_store_for_files(&files);
    let plan = CheckFactPlan {
        vitest_default_exclude_paths: required_files(&root, &config).unwrap(),
        ..Default::default()
    };
    let facts = crate::codebase::check_facts::collect_prepared_check_facts(
        &session,
        &root,
        (files.to_vec(), Vec::new()),
        Vec::new(),
        plan,
        None,
        Arc::clone(&sources),
    );
    assert_eq!(facts.files, files);
    let findings = check_prepared(&root, &config, Some(&facts), &sources, Some(&session)).unwrap();
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, "vitest.external.ts");
    let failures = session.check_failures();
    assert_eq!(failures.len(), 1);
    assert_eq!(failures[0].file.as_deref(), Some(Path::new("malformed.ts")));
    assert_eq!(failures[0].property.as_deref(), Some("test.exclude"));
    assert_eq!(sources.physical_read_count(), 2);
}

#[test]
fn invalid_options_report_the_problem() {
    let root = fixture_root();
    for (config, expected_error) in [
        ("missing-config-option.yml", "requires options.config"),
        (
            "empty-glob.yml",
            "requiredGlobs entries must be nonempty strings",
        ),
        ("nonstring-glob.yml", "requiredGlobs[0]"),
        (
            "unsupported-extension.yml",
            "options.config must name a TypeScript or JavaScript file",
        ),
    ] {
        let error = check_with_files(&root, &load(&root, config), &[]).unwrap_err();
        assert!(
            format!("{error:#}").contains(expected_error),
            "{config}: {error:#}"
        );
    }
}

#[test]
fn missing_config_source_reports_the_selected_file() {
    let root = fixture_root();
    let config = load(&root, "missing-config-file.yml");
    let error = check_with_files(&root, &config, &[]).unwrap_err();
    let failure = error.downcast_ref::<CheckInputFailure>().unwrap();
    assert_eq!(failure.file.as_deref(), Some(Path::new("nonexistent.ts")));
    assert!(failure.message.contains("failed to read"));
}

#[test]
fn out_of_scope_config_paths_are_skipped() {
    let findings = run("excluded-files.yml");
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, "vitest.external.ts");

    let root = fixture_root();
    let config = load(&root, "empty-required-globs.yml");
    assert!(required_files(&root, &config).unwrap().is_empty());
    assert!(check_with_files(&root, &config, &[]).unwrap().is_empty());
}

#[test]
fn file_suppression_covers_a_malformed_config() {
    assert!(run("malformed-suppressed.yml").is_empty());
}
