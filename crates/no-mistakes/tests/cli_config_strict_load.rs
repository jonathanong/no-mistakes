use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/config-v2/strict-root-project/fixture")
}

fn check(root: &Path, config: Option<&str>) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_no-mistakes"));
    command.args(["--jobs", "1", "check", "--format", "json", "--root"]);
    command.arg(root);
    if let Some(config) = config {
        command.args(["--config", config]);
    }
    command.output().unwrap()
}

#[test]
fn invalid_root_and_project_configurations_exit_two_before_analysis() {
    let root = fixture();
    let cases = [
        (
            "4422.yml",
            "rules[0].projects[0]: undeclared project `missing`",
        ),
        ("4423.yml", "projects.example.routes[0]"),
        ("4426.yml", "projects.example.queues.waitScopes[0]"),
        ("4431.yml", "incldue"),
        ("4432.yml", "workflowsDirs"),
        ("4434.yml", "checks.commands[0].include[0]"),
        ("4437.yml", "checks.commands[0].exclude[0]"),
        ("4442.yml", "4442.yml:3:"),
        ("4442.json", "4442.json:2:"),
        ("4443.yml", "rules[0].include[0]"),
        ("4444.yml", "projects.example.root"),
        ("4446.yml", "queues.requiredJobOptions[0]"),
        ("4447.yml", "skipDirectory"),
        ("4448.yml", "incldue"),
        ("4449.yml", "enqueue"),
        ("4451.yml", "routerFields"),
        ("4452.yml", "destinaton"),
        ("4453.yml", "vitset"),
        ("4454.yml", "factory"),
        ("4455.yml", "functons"),
        ("4477.json", "configuration root must be a mapping"),
        ("4477-nested.jsonc", "ci:"),
        ("4477-null.json", "expected struct NoMistakesConfig"),
        ("4477-null.yml", "expected struct NoMistakesConfig"),
        ("4478.yml", "projects.example.postgres.unanalyzableSql"),
        ("4482.yml", "projects.example.trpc.routers[0]"),
        ("null-rules.json", "rules: invalid type: null"),
        ("null-projects.jsonc", "projects: invalid type: null"),
        (
            "null-project-routes.yml",
            "projects.example.routes: invalid type: null",
        ),
    ];
    for (config, expected) in cases {
        let out = check(&root, Some(config));
        let stderr = String::from_utf8(out.stderr).unwrap();
        assert_eq!(out.status.code(), Some(2), "{config}: {stderr}");
        assert!(
            out.stdout.is_empty(),
            "{config}: published a partial report"
        );
        assert!(stderr.contains(config), "{stderr}");
        assert!(stderr.contains(expected), "{stderr}");
    }

    for name in [
        "valid.yml",
        "empty.yml",
        "empty-containers.yml",
        "legacy-sql-policies.yml",
    ] {
        let valid = check(&root, Some(name));
        assert_eq!(valid.status.code(), Some(0), "{name}: {:?}", valid.stderr);
        assert!(serde_json::from_slice::<serde_json::Value>(&valid.stdout).is_ok());
    }
}

#[test]
fn inherited_annotation_policy_uses_normalized_project_value() {
    let root = fixture().join("../../../codebase-analysis/rule-analysis-policy/fixture");
    let output = check(&root, Some("project-annotation-ignore.yml"));
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["rules"], serde_json::json!([]));
    assert!(report["incomplete"].is_null(), "{report}");
}

#[test]
fn strict_config_reports_the_unknown_fragment_key_on_its_own_line() {
    let root = fixture().join("fragment").canonicalize().unwrap();
    let out = check(&root, None);
    assert_eq!(out.status.code(), Some(2));
    assert!(out.stdout.is_empty());
    let stderr = String::from_utf8(out.stderr).unwrap().replace('\\', "/");
    let stderr = stderr.replace(&root.to_string_lossy().replace('\\', "/"), "<root>");
    assert!(stderr.contains("projects/example.yml:3:"), "{stderr}");
    insta::assert_snapshot!("unknown_project_fragment_key", stderr.trim());
}

#[test]
fn server_routes_exits_two_for_invalid_discovered_config() {
    let root = fixture().join("../../../server-ast-routes/malformed-config/fixture");
    let out = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args([
            "--jobs", "1", "server", "routes", "--format", "json", "--root",
        ])
        .arg(root)
        .output()
        .unwrap();
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert_eq!(out.status.code(), Some(2), "{stderr}");
    assert!(out.stdout.is_empty());
    assert!(stderr.contains(".no-mistakes.yml"), "{stderr}");
}

#[test]
fn invalid_rule_scopes_and_yaml_merge_helpers_fail_before_rule_execution() {
    let rule_fixtures = fixture().join("../../../rules");
    for (rule, root, config, field) in [
        (
            "require-storybook-stories",
            "missing",
            "unknown-project.yml",
            "rules[0].projects[0]",
        ),
        (
            "require-storybook-stories",
            "comments",
            "missing-project.yml",
            "rules[0].projects[0]",
        ),
        (
            "no-mistakes-config",
            "merge-key",
            "merge-key.yml",
            "unknown field `shared`",
        ),
    ] {
        let root = rule_fixtures.join(rule).join("fixture").join(root);
        let out = check(&root, Some(&format!("../../configs/{config}")));
        let stderr = String::from_utf8(out.stderr).unwrap();
        assert_eq!(out.status.code(), Some(2), "{stderr}");
        assert!(
            out.stdout.is_empty(),
            "{config}: published a partial report"
        );
        assert!(stderr.contains(field), "{stderr}");
    }
}
