use super::*;
use crate::config::v2::schema::{RuleDef, RuleScope};

fn root(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &crate::test_support::rule_fixture_root(RULE_ID).join(name),
    )
}

fn config(options: &str) -> NoMistakesConfig {
    NoMistakesConfig {
        rules: vec![RuleDef {
            rule: RULE_ID.into(),
            scope: Some(RuleScope::Repository),
            options: serde_yaml::from_str(options).unwrap(),
            ..Default::default()
        }],
        ..Default::default()
    }
}

fn check_fixture(name: &str, config: &NoMistakesConfig) -> Vec<RuleFinding> {
    let root = root(name);
    check_with_files(&root, config, &[root.join(FILE)]).unwrap()
}

#[test]
fn checks_literal_types_and_configured_globs() {
    let mut config = config(
        "minimumDays: 7\nrequireOpenPullRequestsLimit: true\nallowedCooldownExcludes: ['@example/*']",
    );
    config.rules[0].message = Some("Release policy.".into());
    let findings = check_fixture("types", &config);
    let reported: Vec<_> = findings
        .iter()
        .map(|finding| (finding.line, finding.target.as_deref().unwrap()))
        .collect();
    assert_eq!(
        reported,
        [
            (4, "open-pull-requests-limit"),
            (6, "cooldown.default-days"),
            (7, "cooldown.semver-major-days"),
            (8, "cooldown.semver-minor-days"),
            (9, "cooldown.semver-patch-days"),
            (10, "cooldown.exclude"),
            (13, "cooldown.exclude"),
            (14, "cooldown.default-days"),
            (14, "open-pull-requests-limit"),
        ],
    );
    assert!(findings
        .iter()
        .all(|finding| finding.message.starts_with("Release policy.")));
    assert!(findings
        .last()
        .unwrap()
        .message
        .contains("unknown ecosystem"));
}

#[test]
fn omission_and_empty_allowlist_have_different_meanings() {
    let unrestricted = check_fixture("violations", &config("minimumDays: 0"));
    assert_eq!(unrestricted.len(), 1);
    assert_eq!(
        unrestricted[0].target.as_deref(),
        Some("cooldown.default-days")
    );

    let restricted = check_fixture(
        "violations",
        &config("minimumDays: 0\nallowedCooldownExcludes: []"),
    );
    let excludes: Vec<_> = restricted
        .iter()
        .filter(|finding| finding.target.as_deref() == Some("cooldown.exclude"))
        .collect();
    assert_eq!(excludes.len(), 2);
    assert!(excludes[0].message.contains("@example/internal"));
    assert!(excludes[1].message.contains("external-package"));

    let compliant = check_fixture(
        "compliant",
        &config("minimumDays: 7\nrequireOpenPullRequestsLimit: true\nallowedCooldownExcludes: ['@example/*']\nexemptEcosystems: [docker]"),
    );
    assert!(compliant.is_empty());
    assert!(check_fixture("no-updates", &config("minimumDays: 7")).is_empty());
}

#[test]
fn handles_yaml_suppression_and_line_fallback() {
    let config = config("minimumDays: 7");
    let findings = check_fixture("line-suppression", &config);
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].line, 8);
    assert_eq!(
        findings[0].target.as_deref(),
        Some("cooldown.semver-minor-days")
    );
    assert!(check_fixture("file-suppression", &config).is_empty());
    let fallback = check_fixture("no-locations", &config);
    assert_eq!(fallback.len(), 1, "{fallback:?}");
    assert_eq!(fallback[0].line, 1);
}

#[test]
fn malformed_or_unavailable_selected_input_records_a_rule_failure() {
    let config = config("minimumDays: 7");
    for (fixture, expected) in [
        ("malformed", "could not read or parse YAML"),
        ("bad-root", "top-level mapping"),
        ("bad-updates", "updates to be a sequence"),
        ("bad-entry", "updates[1] to be a mapping"),
        ("missing", "could not read or parse YAML"),
    ] {
        let root = root(fixture);
        let files = [root.join(FILE)];
        let sources = rules::source_store_for_files(&files);
        let session = AnalysisSession::disabled();
        assert!(
            check_prepared(&root, &config, &files, &sources, Some(&session))
                .unwrap()
                .is_empty()
        );
        let failures = session.check_failures();
        assert_eq!(failures.len(), 1, "{failures:?}");
        assert_eq!(failures[0].consumer, RULE_ID);
        assert_eq!(failures[0].file.as_deref(), Some(Path::new(FILE)));
        assert!(failures[0].message.contains(expected), "{failures:?}");
        assert!(failures[0]
            .message
            .contains("Repair .github/dependabot.yml"));
        assert_eq!(sources.yaml_parse_count(), 1);
    }

    let root = root("malformed");
    let error = check_with_files(&root, &config, &[root.join(FILE)]).unwrap_err();
    assert_eq!(
        error
            .downcast_ref::<CheckInputFailure>()
            .unwrap()
            .file
            .as_deref(),
        Some(Path::new(FILE))
    );
}

#[test]
fn keeps_selection_inside_visible_repository_scope_and_filters() {
    let mut config = config("minimumDays: 7");
    let input_root = root("ignored-locations");
    let off_path = [
        input_root.join("nested/.github/dependabot.yml"),
        input_root.join(".github/dependabot.yaml"),
    ];
    let sources = rules::source_store_for_files(&off_path);
    assert!(
        check_prepared(&input_root, &config, &off_path, &sources, None)
            .unwrap()
            .is_empty()
    );
    assert_eq!(sources.physical_read_count(), 0);

    let root = root("malformed");
    let files = [root.join(FILE)];
    let sources = rules::source_store_for_files(&files);
    config.rules[0].exclude = vec![FILE.into()];
    assert!(check_prepared(&root, &config, &files, &sources, None)
        .unwrap()
        .is_empty());
    config.rules[0].exclude.clear();
    config.rules[0].enabled = false;
    assert!(check_prepared(&root, &config, &files, &sources, None)
        .unwrap()
        .is_empty());
    assert_eq!(sources.physical_read_count(), 0);
}

#[test]
fn shares_the_parsed_yaml_and_source_between_applications() {
    let mut config = config("minimumDays: 7");
    let mut second = config.rules[0].clone();
    second.name = Some("release delay for all ecosystems".into());
    second.options["minimumDays"] = Value::from(10);
    config.rules.push(second);
    let root = root("violations");
    let files = [root.join(FILE)];
    let sources = rules::source_store_for_files(&files);
    let findings = check_prepared(&root, &config, &files, &sources, None).unwrap();
    assert!(findings
        .iter()
        .any(|finding| finding.message.contains("at least 7")));
    assert!(findings
        .iter()
        .any(|finding| finding.message.contains("at least 10")));
    assert_eq!(sources.physical_read_count(), 1);
    assert_eq!(sources.yaml_parse_count(), 1);
}

#[test]
fn rejects_invalid_rule_options_even_when_input_is_absent() {
    let root = root("missing");
    for (options, expected) in [
        ("{}", "options.minimumDays"),
        ("minimumDays: -1", "options.minimumDays"),
        (
            "minimumDays: 7\nrequireOpenPullRequestsLimit: 1",
            "requireOpenPullRequestsLimit",
        ),
        ("minimumDays: 7\nunknownOption: true", "unknownOption"),
        (
            "minimumDays: 7\nallowedCooldownExcludes: ['[']",
            "options.allowedCooldownExcludes",
        ),
    ] {
        let error = check_with_files(&root, &config(options), &[]).unwrap_err();
        assert!(error.to_string().contains(expected), "{error:#}");
    }
}

#[test]
fn co_configured_pnpm_release_age_policy_reuses_the_dependabot_parse() {
    let pnpm_rule = rules::pnpm_release_age_policy::RULE_ID;
    let root = crate::codebase::ts_resolver::normalize_path(
        &crate::test_support::rule_fixture_root(pnpm_rule).join("fail-dependabot"),
    );
    let files = [
        FILE,
        "pnpm-workspace.yaml",
        "pnpm-lock.yaml",
        "package.json",
    ]
    .map(|path| root.join(path));
    let sources = rules::source_store_for_files(&files);
    let mut config = config("minimumDays: 7");
    config.rules.extend(
        crate::config::v2::load_v2_config(&root, Some(&root.join(".no-mistakes.yml")))
            .unwrap()
            .rules,
    );
    let cooldown = check_prepared(&root, &config, &files, &sources, None).unwrap();
    assert_eq!(cooldown.len(), 1, "{cooldown:?}");
    assert_eq!(cooldown[0].target.as_deref(), Some("cooldown.default-days"));
    assert_eq!(sources.physical_read_count(), 1);

    let pnpm = rules::pnpm_release_age_policy::check_with_files_and_sources(
        &root, &config, &files, &sources,
    )
    .unwrap();
    assert_eq!(pnpm.len(), 1, "{pnpm:?}");
    assert_eq!(pnpm[0].file, FILE);
    assert!(pnpm[0]
        .message
        .contains("is not covered by npm cooldown.exclude"));
    assert_eq!(sources.physical_read_count(), files.len());
    assert_eq!(sources.yaml_parse_count(), 1);
}
