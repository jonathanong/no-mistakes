use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn config(rule: &str, name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules")
        .join(rule)
        .join("config")
        .join(name)
}

const DISPATCH_CONFIG_PATHS: &str =
    "generic_filesystem_rules_run_through_public_dispatch-config-path-references.yml";
const DISPATCH_COMPANIONS: &str =
    "generic_filesystem_rules_run_through_public_dispatch-required-companion-imports.yml";
const DISPATCH_VITEST_PROJECTS: &str =
    "generic_filesystem_rules_run_through_public_dispatch-vitest-project-mapping.yml";
const DISPATCH_PACKAGE_CYCLES: &str =
    "generic_filesystem_rules_run_through_public_dispatch-workspace-package-cycles.yml";
const DISPATCH_CLOSURE: &str =
    "generic_filesystem_rules_run_through_public_dispatch-forbidden-workspace-closure.yml";
const FINITE_SET_GLOB: &str = "finite_set_consistency_glob_coverage_appears_in_cli_json.yml";
const PINNED_HASH_FAIL: &str = "github_actions_pinned_hash_cli_fails_for_tag_ref.yml";
const SCRIPT_EXPRESSIONS_FAIL: &str =
    "github_actions_script_expressions_cli_fails_for_inline_expressions.yml";
const VITEST_CI_PATHS: &str = "vitest_ci_path_coverage_appears_in_cli_json.yml";
const CHECKOUT_FAIL: &str = "github_actions_checkout_credentials_cli_fails_for_a_write_token.yml";

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_no-mistakes"))
}

fn rules_dir(parts: &[&str]) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../test-cases/rules");
    for part in parts {
        path.push(part);
    }
    no_mistakes::codebase::ts_resolver::normalize_path(&path)
}

fn fixture(category: &str, scenario: &str) -> PathBuf {
    rules_dir(&[category, "fixture", scenario])
}

fn rule_fixture(category: &str) -> PathBuf {
    rules_dir(&[category, "fixture"])
}

fn rule_fixture_scenario(category: &str, scenario: &str) -> PathBuf {
    rules_dir(&[category, scenario])
}

fn filesystem_findings(
    root: &Path,
    config: &Path,
) -> Vec<no_mistakes::codebase::rules::RuleFinding> {
    no_mistakes::codebase::rules::run_filesystem_rules(root, Some(config)).unwrap()
}

fn check(root: &Path, config: &Path) -> Output {
    Command::new(bin())
        .args(["check", "--root"])
        .arg(root)
        .arg("--config")
        .arg(config)
        .output()
        .unwrap()
}

fn stdout(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

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
    let out = check(
        &root,
        &config("github-actions-pinned-hash", PINNED_HASH_FAIL),
    );
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
fn github_actions_script_expressions_flags_run_and_script_bodies() {
    let root = fixture("github-actions-script-expressions", "fail");
    let findings = no_mistakes::codebase::rules::run_filesystem_rules(&root, None).unwrap();
    let body = format!("{findings:?}");

    assert!(!findings.is_empty(), "expected findings");
    assert!(body.contains("github-actions-script-expressions"), "{body}");
    assert!(body.contains("github.ref"), "{body}");
    assert!(body.contains("ci.yml"), "{body}");
    assert!(body.contains("action.yml"), "{body}");
}

#[test]
fn github_actions_script_expressions_passes_for_env_and_constants() {
    let root = fixture("github-actions-script-expressions", "pass");
    let findings = no_mistakes::codebase::rules::run_filesystem_rules(&root, None).unwrap();
    assert!(findings.is_empty(), "unexpected findings: {findings:?}");
}

#[test]
fn github_actions_script_expressions_cli_fails_for_inline_expressions() {
    let root = fixture("github-actions-script-expressions", "fail");
    let out = check(
        &root,
        &config("github-actions-script-expressions", SCRIPT_EXPRESSIONS_FAIL),
    );
    let body = stdout(&out);

    assert!(!out.status.success(), "expected exit 1: {body}");
    assert!(body.contains("github-actions-script-expressions"), "{body}");
    assert!(body.contains("github.actor"), "{body}");
}

#[test]
fn generic_filesystem_rules_run_through_public_dispatch() {
    let config_paths = filesystem_findings(
        &rule_fixture("config-path-references"),
        &config("config-path-references", DISPATCH_CONFIG_PATHS),
    );
    assert!(format!("{config_paths:?}").contains("missing.json"));
    let companions = filesystem_findings(
        &rule_fixture("required-companion-imports"),
        &config("required-companion-imports", DISPATCH_COMPANIONS),
    );
    assert!(format!("{companions:?}").contains("@/components/Card"));
    let vitest_projects = filesystem_findings(
        &rule_fixture("vitest-project-mapping"),
        &config("vitest-project-mapping", DISPATCH_VITEST_PROJECTS),
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
        &config("workspace-package-cycles", DISPATCH_PACKAGE_CYCLES),
    );
    assert!(format!("{package_cycles:?}").contains("@x/api -> @x/domain -> @x/api"));
    let closure_root = rule_fixture_scenario("forbidden-workspace-closure", "transitive-workspace");
    let closure = filesystem_findings(
        &closure_root,
        &config("forbidden-workspace-closure", DISPATCH_CLOSURE),
    );
    assert!(format!("{closure:?}").contains("@acme/app -> @acme/domain -> @acme/secret"));
}

#[test]
fn finite_set_consistency_glob_coverage_appears_in_cli_json() {
    let root = rule_fixture("finite-set-consistency");
    let config_path = config("finite-set-consistency", FINITE_SET_GLOB);
    let out = Command::new(bin())
        .args(["check", "--root"])
        .arg(root)
        .arg("--config")
        .arg(&config_path)
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
fn finite_set_call_sources_under_skipped_directories_stay_out_of_other_checks() {
    let root = no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/rules/finite-set-consistency/call-literals/supplemental-skipped"),
    );
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

    assert!(!out.status.success(), "the finite-set mismatch should fail");
    assert!(json["rules"].as_array().unwrap().iter().any(|finding| {
        finding["rule"] == "finite-set-consistency" && finding["target"] == "missing"
    }));
    // The skipped generated source is also a graph root. Its configured
    // call-site demand must not replace graph import facts with a sparse view.
    assert!(json["rules"].as_array().unwrap().iter().any(|finding| {
        finding["rule"] == "forbidden-dependencies" && finding["target"] == "src/lazy.mts"
    }));
    // `generated/` is a configured finite-set input, but it is outside the
    // dynamic-import check's filesystem scope.
    assert!(json["rules"]
        .as_array()
        .unwrap()
        .iter()
        .all(|finding| { finding["rule"] != "test-no-unmocked-dynamic-imports" }));
}

#[test]
fn vitest_ci_path_coverage_appears_in_cli_json() {
    let root = rule_fixture_scenario("vitest-ci-path-coverage", "fixture");
    let out = check(&root, &config("vitest-ci-path-coverage", VITEST_CI_PATHS));
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
            && finding.import.as_deref() == Some("msw")),
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

#[cfg(unix)]
fn finite_set_symlink_fixture(name: &str) -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/rules/finite-set-consistency")
            .join(name),
    )
}

#[cfg(unix)]
#[test]
fn finite_set_path_regex_matches_directory_and_file_target_symlinks() {
    let root = finite_set_symlink_fixture("path-regex-directory-symlink");
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
    let rules = json["rules"].as_array().unwrap();

    assert!(out.status.success(), "{body}");
    assert!(
        rules
            .iter()
            .all(|finding| finding["rule"] != "finite-set-consistency"),
        "{body}"
    );
}

#[cfg(unix)]
#[test]
fn finite_set_path_regex_matches_hidden_skill_directory_symlinks() {
    let root = finite_set_symlink_fixture("path-regex-skill-symlinks");
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
    let rules = json["rules"].as_array().unwrap();

    assert!(out.status.success(), "{body}");
    assert!(
        rules
            .iter()
            .all(|finding| finding["rule"] != "finite-set-consistency"),
        "{body}"
    );
}

#[cfg(unix)]
#[test]
fn finite_set_path_regex_reports_skill_file_without_matching_symlink() {
    let root = finite_set_symlink_fixture("path-regex-skill-symlinks-mismatch");
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

    assert!(!out.status.success(), "{body}");
    assert!(
        json["rules"].as_array().unwrap().iter().any(|finding| {
            finding["rule"] == "finite-set-consistency" && finding["target"] == "orphan-skill"
        }),
        "{body}"
    );
}

#[cfg(unix)]
#[test]
fn finite_set_path_regex_matches_broken_symlink_path_existence() {
    let root = finite_set_symlink_fixture("path-regex-broken-symlink");
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

    assert!(out.status.success(), "{body}");
    assert!(
        json["rules"]
            .as_array()
            .unwrap()
            .iter()
            .all(|finding| finding["rule"] != "finite-set-consistency"),
        "{body}"
    );
}

#[test]
fn github_actions_checkout_credentials_cli_fails_for_a_write_token() {
    let root = fixture("github-actions-checkout-credentials", "fail");
    let out = check(
        &root,
        &config("github-actions-checkout-credentials", CHECKOUT_FAIL),
    );
    let body = stdout(&out);
    assert!(!out.status.success(), "{body}");
    assert!(
        body.contains("github-actions-checkout-credentials"),
        "{body}"
    );
}
