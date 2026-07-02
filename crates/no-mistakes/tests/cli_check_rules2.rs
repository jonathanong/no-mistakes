use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_no-mistakes"))
}

fn fixture(category: &str, scenario: &str) -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules")
            .join(category)
            .join("fixture")
            .join(scenario),
    )
}

fn rule_fixture(category: &str) -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules")
            .join(category)
            .join("fixture"),
    )
}

fn rule_fixture_scenario(category: &str, scenario: &str) -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules")
            .join(category)
            .join(scenario),
    )
}

fn filesystem_findings(root: &Path, yaml: &str) -> Vec<no_mistakes::codebase::rules::RuleFinding> {
    let config = tempfile::Builder::new().suffix(".yml").tempfile().unwrap();
    std::fs::write(config.path(), yaml).unwrap();
    no_mistakes::codebase::rules::run_filesystem_rules(root, Some(config.path())).unwrap()
}

fn check_fixture_config(root: &PathBuf, name: &str) -> Output {
    let yaml = std::fs::read_to_string(root.join(name)).unwrap();
    let config = tempfile::Builder::new().suffix(".yml").tempfile().unwrap();
    std::fs::write(config.path(), &yaml).unwrap();
    Command::new(bin())
        .args(["check", "--root"])
        .arg(root)
        .arg("--config")
        .arg(config.path())
        .output()
        .unwrap()
}

fn stdout(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

// ── github-actions-pinned-hash ───────────────────────────────────────────────

#[test]
fn github_actions_pinned_hash_fails_for_tag_ref() {
    let root = fixture("github-actions-pinned-hash", "fail");
    let findings = no_mistakes::codebase::rules::run_filesystem_rules(&root, None).unwrap();
    let body = format!("{findings:?}");

    assert!(!findings.is_empty(), "expected findings");
    assert!(body.contains("github-actions-pinned-hash"), "{body}");
    assert!(body.contains("ci.yml"), "{body}");
}

#[test]
fn github_actions_pinned_hash_passes_for_pinned_workflows() {
    let root = fixture("github-actions-pinned-hash", "pass");
    let findings = no_mistakes::codebase::rules::run_filesystem_rules(&root, None).unwrap();
    assert!(findings.is_empty(), "unexpected findings: {findings:?}");
}

#[test]
fn github_actions_pinned_hash_cli_fails_for_tag_ref() {
    let root = fixture("github-actions-pinned-hash", "fail");
    let out = check_fixture_config(&root, ".no-mistakes.yml");
    let body = stdout(&out);

    assert!(!out.status.success(), "expected exit 1");
    assert!(body.contains("github-actions-pinned-hash"), "{body}");
    assert!(body.contains("ci.yml"), "{body}");
}

#[test]
fn github_actions_pinned_hash_passes_for_local_actions() {
    let root = fixture("github-actions-pinned-hash", "local-action-pass");
    let findings = no_mistakes::codebase::rules::run_filesystem_rules(&root, None).unwrap();
    assert!(findings.is_empty(), "unexpected findings: {findings:?}");
}

#[test]
fn generic_filesystem_rules_run_through_public_dispatch() {
    let config_paths = filesystem_findings(
        &rule_fixture("config-path-references"),
        r#"
rules:
  - rule: config-path-references
    scope: repository
    options:
      files: [config/app.yml]
      keys: [paths.required]
      baseDir: root
"#,
    );
    assert!(format!("{config_paths:?}").contains("missing.json"));

    let companions = filesystem_findings(
        &rule_fixture("required-companion-imports"),
        r#"
rules:
  - rule: required-companion-imports
    scope: repository
    options:
      sourceDirs: [src/components]
      directChildOnly: true
      sourceExtensions: [.tsx]
      excludeBasenames: [Internal.tsx, Button.stories.tsx, Card.stories.tsx]
      companionGlobs: ["{sourceDir}/{sourceStem}.stories.tsx"]
      specifierTemplate: "@/components/{sourceStem}"
      stripSourcePrefix: src/
"#,
    );
    assert!(format!("{companions:?}").contains("@/components/Card"));

    let vitest_projects = filesystem_findings(
        &rule_fixture("vitest-project-mapping"),
        r#"
tests:
  vitest:
    configs: vitest.config.mts
rules:
  - rule: vitest-project-mapping
    scope: repository
"#,
    );
    assert!(format!("{vitest_projects:?}").contains("src/unmapped.test.ts"));

    let vitest_ci_root = rule_fixture_scenario("vitest-ci-path-coverage", "fixture");
    let vitest_ci = no_mistakes::codebase::rules::run_filesystem_rules(
        &vitest_ci_root,
        Some(&vitest_ci_root.join(".no-mistakes.yml")),
    )
    .unwrap();
    assert!(format!("{vitest_ci:?}").contains("ts-shared/utils/index.mts"));

    let package_cycles = filesystem_findings(
        &rule_fixture_scenario("workspace-package-cycles", "cycle"),
        r#"
rules:
  - rule: workspace-package-cycles
    scope: repository
"#,
    );
    assert!(format!("{package_cycles:?}").contains("@x/api -> @x/domain -> @x/api"));
}

#[test]
fn finite_set_consistency_glob_coverage_appears_in_cli_json() {
    let root = rule_fixture("finite-set-consistency");
    let config = tempfile::Builder::new().suffix(".yml").tempfile().unwrap();
    std::fs::write(
        config.path(),
        r#"
rules:
  - rule: finite-set-consistency
    scope: repository
    options:
      sets:
        - name: registry
          file: src/types.ts
          kind: ts-const-array-property
          target: FIRST_PARTY_EXEMPTIONS
          property: name
        - name: dependabotGlobs
          file: .github/dependabot.yml
          kind: yaml-sequence
          key: updates.0.cooldown.exclude
        - name: names
          file: src/types.ts
          kind: ts-array-literal
          target: FIRST_PARTY_NAMES
        - name: docsMentions
          file: docs/dependency-updates.md
          kind: markdown-table-code-cells
      comparisons:
        - left: registry
          right: dependabotGlobs
          mode: glob-coverage
        - left: names
          right: docsMentions
"#,
    )
    .unwrap();

    let out = Command::new(bin())
        .args(["check", "--root"])
        .arg(root)
        .arg("--config")
        .arg(config.path())
        .args(["--format", "json"])
        .output()
        .unwrap();
    let body = stdout(&out);

    assert!(!out.status.success(), "expected finite-set finding");
    assert!(body.contains("finite-set-consistency"), "{body}");
    assert!(body.contains("@acme/docs"), "{body}");
    assert!(
        body.contains("no glob in dependabotGlobs covers it"),
        "{body}"
    );
}

#[test]
fn vitest_ci_path_coverage_appears_in_cli_json() {
    let root = rule_fixture_scenario("vitest-ci-path-coverage", "fixture");
    let out = check_fixture_config(&root, ".no-mistakes.yml");
    let body = stdout(&out);

    assert!(!out.status.success(), "expected vitest CI path finding");
    assert!(body.contains("vitest-ci-path-coverage"), "{body}");
    assert!(body.contains("ts-shared/utils/index.mts"), "{body}");

    let json = Command::new(bin())
        .args(["check", "--root"])
        .arg(&root)
        .arg("--config")
        .arg(root.join(".no-mistakes.yml"))
        .args(["--format", "json"])
        .output()
        .unwrap();
    let json_body = stdout(&json);
    let json_value: serde_json::Value =
        serde_json::from_str(&json_body).expect("stdout should be json");
    assert!(!json.status.success(), "expected json check failure");
    assert!(
        json_value.to_string().contains("vitest-ci-path-coverage"),
        "{json_value}"
    );
    assert!(
        json_value.to_string().contains("ts-shared/utils/index.mts"),
        "{json_value}"
    );
}

#[test]
fn migrated_generic_content_rules_run_through_filesystem_dispatch() {
    let integration_root = rule_fixture("integration-test-no-mocks");
    let integration = no_mistakes::codebase::rules::run_filesystem_rules(
        &integration_root,
        Some(&integration_root.join(".no-mistakes.yml")),
    )
    .unwrap();
    let integration_body = format!("{integration:?}");
    assert!(
        integration.iter().any(|finding| finding.rule
            == no_mistakes::codebase::rules::INTEGRATION_TEST_NO_MOCKS
            && finding.file == "integration-tests/web-api/mocked.test.mts"
            && finding.import.as_deref() == Some("vi.mock")),
        "{integration_body}"
    );
    assert!(
        integration
            .iter()
            .any(|finding| finding.import.as_deref() == Some("msw")),
        "{integration_body}"
    );
    assert!(
        !integration_body.contains("backend/service.mock.test.mts"),
        "{integration_body}"
    );

    let email_root = rule_fixture("test-email-domain-policy");
    let email = no_mistakes::codebase::rules::run_filesystem_rules(
        &email_root,
        Some(&email_root.join(".no-mistakes.yml")),
    )
    .unwrap();
    let email_body = format!("{email:?}");
    assert!(
        email.iter().any(|finding| finding.rule
            == no_mistakes::codebase::rules::TEST_EMAIL_DOMAIN_POLICY
            && finding.file == "backend/services/emails/send.test.mts"
            && finding.target.as_deref() == Some("example.com")),
        "{email_body}"
    );
    assert!(
        email
            .iter()
            .any(|finding| finding.file == "email-templates/__snapshots__/login.txt"),
        "{email_body}"
    );
    assert!(!email_body.contains("source.mts"), "{email_body}");

    let markdown_root = rule_fixture("markdown-link-display-text");
    let markdown = no_mistakes::codebase::rules::run_filesystem_rules(
        &markdown_root,
        Some(&markdown_root.join(".no-mistakes.yml")),
    )
    .unwrap();
    let markdown_body = format!("{markdown:?}");
    assert_eq!(markdown.len(), 2, "{markdown_body}");
    assert!(
        markdown.iter().any(|finding| {
            finding.rule == no_mistakes::codebase::rules::MARKDOWN_LINK_DISPLAY_TEXT
                && finding.import.as_deref() == Some("SOURCE-STORIES.md")
                && finding.target.as_deref() == Some("news-story-clusters.md")
        }),
        "{markdown_body}"
    );
}

#[test]
fn migrated_generic_content_rules_appear_in_cli_json() {
    let root = rule_fixture("markdown-link-display-text");
    let out = Command::new(bin())
        .args(["check", "--root"])
        .arg(&root)
        .arg("--config")
        .arg(root.join(".no-mistakes.yml"))
        .args(["--format", "json"])
        .output()
        .unwrap();
    let body = stdout(&out);
    let json: serde_json::Value = serde_json::from_str(&body).expect("stdout should be json");

    assert!(!out.status.success(), "expected markdown link finding");
    assert!(json["rules"].as_array().unwrap().iter().any(|finding| {
        finding["rule"] == "markdown-link-display-text"
            && finding["file"] == "docs/bad.md"
            && finding["target"] == "news-story-clusters.md"
    }));
}

#[test]
fn vitest_ci_path_coverage_supports_no_mistakes_suppression() {
    let root = rule_fixture_scenario("vitest-ci-path-coverage", "suppressed");
    let findings = no_mistakes::codebase::rules::run_filesystem_rules(
        &root,
        Some(&root.join(".no-mistakes.yml")),
    )
    .unwrap();

    assert!(findings.is_empty(), "{findings:#?}");
}
