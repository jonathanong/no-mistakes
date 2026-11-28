use super::check_json_snapshot::{
    assert_saved_fixture_checked, assert_saved_fixture_report, rules_path,
};
use no_mistakes::codebase::rules::rule_ids::configurable_rule_ids;
use serde_yaml::Value;
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::process::Command;

#[test]
fn invalid_rule_configuration_aborts_before_analysis() {
    let root = rules_path("options-audit", "fixture", "issue-4110");
    let cases = [
        ("unknown", root.clone(), root.join("unknown.yml")),
        ("unknown-map", root.clone(), root.join("unknown-map.yml")),
        (
            "unknown-override",
            root.clone(),
            root.join("unknown-override.yml"),
        ),
        (
            "unknown-project",
            root.clone(),
            root.join("unknown-project.yml"),
        ),
        ("unknown-jsonc", root.clone(), root.join("unknown.jsonc")),
        (
            "invalid-options",
            root.clone(),
            root.join("invalid-options.yml"),
        ),
        (
            "invalid-values",
            root.clone(),
            root.join("invalid-values.yml"),
        ),
        (
            "invalid-declared-values",
            root.clone(),
            root.join("invalid-declared-values.yml"),
        ),
        (
            "semantic-errors",
            root.clone(),
            root.join("semantic-errors.yml"),
        ),
        (
            "swift-test-policy-declared",
            root.clone(),
            root.join("swift-test-policy-declared.yml"),
        ),
        ("fragment", root.join("fragment"), PathBuf::new()),
    ];
    let result = assert_saved_fixture_checked(
        "invalid_rule_configuration",
        &cases,
        &[],
        false,
        &[],
        &mut redact_invalid_declared_dependency_tails,
    );
    for (label, _, _) in cases {
        assert_eq!(result[label]["exitCode"], 2, "{label}: {result}");
        assert!(result[label]["report"].is_null(), "{label}: {result}");
    }
    let declared = result["invalid-declared-values"]["stderr"]
        .as_str()
        .unwrap_or("");
    for id in [
        "agents-md-max-size",
        "ast-grep-ignore-freshness",
        "bash-empty-array-nounset",
        "config-path-references",
        "coverage-producer-assertion-owner",
        "csp-allowed-origins",
        "dependabot-cooldown-policy",
        "doc-consistency",
        "file-extension-policy",
        "integration-test-no-mocks",
        "lockfile-allowlist",
        "playwright-no-serial",
        "pnpm-release-age-policy",
        "production-dependency-declarations",
        "production-exports-only-used-by-tests",
        "server-route-client-boundary",
        "shellcheck-runner",
        "strict-package-layout",
        "structured-config-policy",
        "unique-exports",
        "vite-optimize-deps-nested-imports",
        "workspace-package-cycles",
        "zsh-readonly-parameter",
    ] {
        assert!(
            declared.contains(id),
            "missing invalid declared value for {id}: {declared}"
        );
    }
    for owned in [
        "error parsing glob",
        "regex parse error",
        "unclosed character class",
        "unclosed group",
    ] {
        assert!(
            !declared.contains(owned),
            "dependency-owned text {owned} stayed in the snapshot: {declared}"
        );
    }
    let swift_declared = result["swift-test-policy-declared"]["stderr"]
        .as_str()
        .expect("invalid declared values");
    for marker in [
        "swift-no-raw-print",
        "swift-viewmodel-main-actor",
        "test-no-dependency-pins",
        "test-no-live-repository-scan: fixtureDirs",
    ] {
        assert!(
            swift_declared.contains(marker),
            "{marker} missing: {swift_declared}"
        );
    }
}

#[test]
fn valid_declared_option_values_still_run() {
    let root = rules_path("options-audit", "fixture", "valid-declared-values");
    let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["--lock-timeout", "0", "--jobs", "1", "check", "--root"])
        .arg(&root)
        .arg("--config")
        .arg(root.join(".no-mistakes.yml"))
        .args(["--format", "json"])
        .output()
        .unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert_eq!(output.status.code(), Some(0), "{stderr}");
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).expect(&stderr);
    assert!(report.is_object(), "{report}");
}

#[test]
fn invalid_nested_rule_options_abort_before_analysis() {
    let root = rules_path("options-audit", "fixture", "issue-4575");
    let labels: Vec<_> = (4530..=4558).map(|issue| issue.to_string()).collect();
    let mut cases: Vec<_> = labels
        .iter()
        .map(|label| {
            (
                label.as_str(),
                root.clone(),
                root.join(format!("{label}.yml")),
            )
        })
        .collect();
    cases.push((
        "literal-selector",
        root.clone(),
        root.join("literal-selector.yml"),
    ));
    cases.push((
        "malformed-normalized-selector",
        root.clone(),
        root.join("malformed-normalized-selector.yml"),
    ));
    let result = assert_saved_fixture_report("invalid_nested_rule_options", &cases, &[], &[]);
    for label in labels {
        assert_eq!(result[&label]["exitCode"], 2, "{label}: {result}");
        assert!(result[&label]["report"].is_null(), "{label}: {result}");
    }
    assert_eq!(result["literal-selector"]["exitCode"], 0);
    assert_eq!(result["malformed-normalized-selector"]["exitCode"], 2);
    assert!(result["malformed-normalized-selector"]["stderr"]
        .as_str()
        .unwrap()
        .contains("options.roots[0]: invalid glob"));
}

#[test]
fn invalid_declared_option_values_abort_before_analysis() {
    let root = rules_path("options-audit", "fixture", "issue-4654");
    let cases = [(
        "invalid-declared",
        root.clone(),
        root.join("invalid-declared.yml"),
    )];
    let result = assert_saved_fixture_report("invalid_declared_option_values", &cases, &[], &[]);
    let case = &result["invalid-declared"];
    let stderr = case["stderr"].as_str().unwrap_or("");
    assert_eq!(case["exitCode"], 2, "{stderr}");
    assert!(case["report"].is_null(), "{stderr}");
}

#[test]
fn valid_declared_option_values_match_snapshot() {
    let root = rules_path("options-audit", "fixture", "issue-4654");
    let cases = [(
        "valid-declared",
        root.clone(),
        root.join("valid-declared.yml"),
    )];
    let result = assert_saved_fixture_report("valid_declared_option_values", &cases, &[], &[]);
    let case = &result["valid-declared"];
    assert_eq!(case["exitCode"], 0, "{case}");
    assert!(case["report"].is_object(), "{case}");
}

#[test]
fn every_registered_rule_rejects_an_unknown_option() {
    let root = rules_path("options-audit", "fixture", "issue-4110");
    let path = root.join("all-rules.yml");
    let yaml: Value = serde_yaml::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    let rules = yaml["rules"].as_sequence().unwrap();
    let declared: BTreeSet<_> = rules
        .iter()
        .map(|rule| {
            assert_eq!(rule["options"]["bogusOption"].as_bool(), Some(true));
            rule["rule"].as_str().unwrap()
        })
        .collect();
    let registered: BTreeSet<_> = configurable_rule_ids().collect();
    assert!(!registered.is_empty());
    assert_eq!(
        declared, registered,
        "update the saved fixture when registering a rule"
    );
    assert_eq!(rules.len(), registered.len());

    let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["--lock-timeout", "0", "--jobs", "1", "check", "--root"])
        .arg(&root)
        .arg("--config")
        .arg(&path)
        .args(["--format", "json"])
        .output()
        .unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert_eq!(output.status.code(), Some(2), "{stderr}");
    assert!(output.stdout.is_empty(), "{stderr}");
    for id in registered {
        let diagnostic = stderr
            .lines()
            .find(|line| line.contains(&format!("`{id}`")));
        assert!(
            diagnostic.is_some_and(|line| line.contains("bogusOption") && line.contains("all-rules.yml:")),
            "missing bogusOption diagnostic for {id}: {stderr}"
        );
    }
}

#[test]
fn declared_option_values_reject_invalid_and_run_valid() {
    let root = rules_path("options-audit", "fixture", "declared-invalid-options");
    let cases = [
        (
            "invalid",
            root.clone(),
            root.join("invalid-declared-options.yml"),
        ),
        (
            "valid",
            root.clone(),
            root.join("valid-declared-options.yml"),
        ),
    ];
    let result = assert_saved_fixture_report("declared_option_values", &cases, &[], &[]);
    assert_eq!(result["invalid"]["exitCode"], 2, "{}", result["invalid"]);
    assert!(
        result["invalid"]["report"].is_null(),
        "{}",
        result["invalid"]
    );
    let stderr = result["invalid"]["stderr"].as_str().unwrap();
    for (rule, option) in [
        ("nextjs-redirect-destinations", "includeRewrites"),
        ("rust-max-lines-per-file", "srcMax"),
        ("rust-no-inline-allows", "testMax"),
        ("rust-no-inline-tests", "excludes"),
        ("postgres-array-columns", "allowElementTypes"),
        ("postgres-bounded-statements", "statements"),
        ("postgres-conflict-ordering", "include"),
        ("postgres-constraint-validate", "sqlInclude"),
        ("postgres-duplicate-function-body", "minClusterSize"),
        ("postgres-explain-write-rollback", "sqlInclude"),
        ("postgres-explicit-columns", "maxColumns"),
        ("postgres-finite-text-columns", "columnTypes"),
        ("postgres-fk-index", "allowedColumns"),
        ("postgres-generated-column-predicates", "functions"),
        ("postgres-idempotent-insert", "include"),
        ("postgres-identifier-length", "maxBytes"),
        ("postgres-json-relation-ids", "idKeyPattern"),
        ("postgres-key-column-types", "allowedTypes"),
        ("postgres-lock-ordering", "include"),
        ("postgres-lock-written-rows-only", "not-a-check"),
        ("postgres-materialized-view-shape", "windowFunctions"),
        ("postgres-no-transformed-uuid-predicates", "sqlInclude"),
        ("postgres-redundant-index", "allowedIndexes"),
        ("postgres-require-fk-on-delete", "sqlInclude"),
        ("postgres-require-named-constraints", "sqlInclude"),
        ("postgres-require-query-annotation", "sqlLiteralHelpers"),
        ("postgres-required-comments", "objects"),
        ("postgres-sql-statement-policy", "bannedStatements"),
        ("postgres-status-with-lifecycle-timestamps", "statusColumns"),
    ] {
        let diagnostic = stderr.lines().find(|line| line.contains(rule));
        assert!(
            diagnostic.is_some_and(|line| line.contains(option)),
            "{rule} options.{option} was accepted:\n{stderr}"
        );
    }
    assert_eq!(result["valid"]["exitCode"], 0, "{}", result["valid"]);
    assert!(
        result["valid"]["report"]["rules"]
            .as_array()
            .is_some_and(Vec::is_empty),
        "{}",
        result["valid"]
    );
}

const DECLARED_OPTION_RULES: &[&str] = &[
    "banned-paths",
    "banned-renamed-files",
    "csharp-max-lines-per-file",
    "csharp-no-async-void-delegate",
    "forbidden-dependencies",
    "forbidden-lint-suppressions",
    "forbidden-workspace-closure",
    "github-actions-action-input-policy",
    "github-actions-action-timeout-pair",
    "github-actions-cache-key-policy",
    "github-actions-cache-setup-policy",
    "github-actions-composite-step-schema",
    "github-actions-job-timeouts",
    "github-actions-local-action-checkout",
    "github-actions-max-inline-script-lines",
    "github-actions-no-required-pr-path-filters",
    "github-actions-pinned-hash",
    "github-actions-script-expressions",
    "github-actions-step-timeouts",
    "github-actions-test-timeout-literals",
    "github-actions-vitest-project-selection",
    "markdown-child-links",
    "markdown-eval-tests",
    "markdown-link-display-text",
    "markdown-structure-budget",
];

#[test]
fn saved_declared_option_values_fail_closed_and_valid_values_run() {
    let root = rules_path("options-audit", "fixture", "issue-4642");
    let cases = [
        (
            "invalid-declared",
            root.clone(),
            root.join("invalid-declared.yml"),
        ),
        (
            "valid-declared",
            root.clone(),
            root.join("valid-declared.yml"),
        ),
    ];
    let result = assert_saved_fixture_report("declared_option_values_4642", &cases, &[], &[]);
    assert_eq!(result["invalid-declared"]["exitCode"], 2, "{result}");
    assert!(result["invalid-declared"]["report"].is_null(), "{result}");
    let stderr = result["invalid-declared"]["stderr"].as_str().unwrap();
    for rule in DECLARED_OPTION_RULES {
        assert!(
            stderr.contains(rule),
            "missing invalid declared option for {rule}: {stderr}"
        );
    }
    assert_eq!(result["valid-declared"]["exitCode"], 0, "{result}");
    assert!(
        result["valid-declared"]["report"]["rules"]
            .as_array()
            .is_some_and(Vec::is_empty),
        "{result}"
    );
}

// Glob and regex parsers own the text after these markers.
const DEPENDENCY_TAIL_MARKERS: [&str; 3] = ["invalid regex:", "invalid glob", "invalid type:"];

fn redact_invalid_declared_dependency_tails(document: &mut serde_json::Value) {
    let Some(stderr) = document["invalid-declared-values"]["stderr"]
        .as_str()
        .map(str::to_owned)
    else {
        return;
    };
    document["invalid-declared-values"]["stderr"] =
        serde_json::Value::String(strip_dependency_owned_tails(&stderr));
}

fn strip_dependency_owned_tails(stderr: &str) -> String {
    let mut redacted = String::with_capacity(stderr.len());
    let mut skip_continuation = false;
    for line in stderr.split_inclusive('\n') {
        let body = line.trim_end_matches(['\r', '\n']);
        if skip_continuation && !config_diagnostic_line(body) {
            continue;
        }
        let end = earliest_dependency_tail(body);
        skip_continuation = end.is_some();
        redacted.push_str(&body[..end.unwrap_or(body.len())]);
        redacted.push_str(&line[body.len()..]);
    }
    redacted
}

fn earliest_dependency_tail(line: &str) -> Option<usize> {
    DEPENDENCY_TAIL_MARKERS
        .into_iter()
        .filter_map(|marker| line.find(marker).map(|start| start + marker.len()))
        .min()
}

fn config_diagnostic_line(line: &str) -> bool {
    let rest = line.strip_prefix("error: ").unwrap_or(line);
    let Some((path, after)) = rest.split_once(':') else {
        return false;
    };
    let file_name = path.rsplit(['/', '\\']).next().unwrap_or(path);
    let config_file = [".yml", ".yaml", ".json", ".jsonc"]
        .iter()
        .any(|suffix| file_name.ends_with(suffix));
    config_file && after.starts_with(|character: char| character.is_ascii_digit())
}

#[test]
fn declared_option_values_fail_before_analysis() {
    let root = rules_path("options-audit", "fixture", "declared-option-values");
    let mut configs: Vec<_> = std::fs::read_dir(&root)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "yml"))
        .collect();
    configs.sort();
    let cases: Vec<_> = configs
        .iter()
        .map(|config| {
            let name = config.file_stem().unwrap().to_str().unwrap();
            (name, root.clone(), config.clone())
        })
        .collect();
    let report =
        assert_saved_fixture_report("declared_vitest_tsconfig_option_values", &cases, &[], &[]);
    assert_eq!(report["valid"]["exitCode"], 0, "{}", report["valid"]);
    for (name, _, _) in cases {
        if name == "valid" {
            continue;
        }
        let result = &report[name];
        assert_eq!(result["exitCode"], 2, "{name}: {result}");
        assert!(result["report"].is_null(), "{name}: {result}");
        let stderr = result["stderr"].as_str().unwrap();
        assert!(stderr.contains(&format!("{name}.yml:")), "{result}");
        assert!(stderr.contains(&format!("`{name}`")), "{result}");
    }
}
