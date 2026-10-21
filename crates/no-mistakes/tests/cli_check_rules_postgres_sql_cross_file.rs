use std::path::PathBuf;
use std::process::{Command, Output};

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_no-mistakes"))
}

fn fixture(rule: &str, scenario: &str) -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules")
            .join(rule)
            .join("fixture")
            .join(scenario),
    )
}

fn check(root: &PathBuf) -> Output {
    Command::new(bin())
        .args(["check", "--root"])
        .arg(root)
        .args(["--config", ".no-mistakes.yml"])
        .output()
        .unwrap()
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

#[test]
fn mutable_registry_fails_for_cross_file_member_writes() {
    let root = fixture("postgres-no-mutable-sql-registry", "fail-member-write");
    let out = check(&root);
    let body = stdout(&out);
    assert!(!out.status.success(), "{body}");
    assert!(body.contains("postgres-no-mutable-sql-registry"), "{body}");
    assert!(body.contains("query.ts"), "{body}");
    assert!(body.contains("tables.ts"), "{body}");
}

#[test]
fn unbounded_identifier_fails_for_process_env() {
    let root = fixture("postgres-no-unbounded-sql-identifier", "fail-env");
    let out = check(&root);
    let body = stdout(&out);
    assert!(!out.status.success(), "{body}");
    assert!(
        body.contains("postgres-no-unbounded-sql-identifier"),
        "{body}"
    );
    assert!(body.contains("process.env"), "{body}");
}

#[test]
fn switch_validator_and_literal_registry_pass() {
    for (rule, scenario) in [
        ("postgres-no-unbounded-sql-identifier", "pass-switch"),
        ("postgres-no-mutable-sql-registry", "pass-literal-registry"),
    ] {
        let root = fixture(rule, scenario);
        let out = check(&root);
        assert!(out.status.success(), "{rule}/{scenario}: {}", stdout(&out));
    }
}

fn exact_locations(rule: &str, scenario: &str, expected: &[(u64, &str)]) {
    let out = Command::new(bin())
        .args(["check", "--format", "json", "--root"])
        .arg(fixture(rule, scenario))
        .args(["--config", ".no-mistakes.yml"])
        .output()
        .unwrap();
    let report: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let findings: Vec<_> = report["rules"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|finding| finding["rule"] == rule)
        .collect();
    assert_eq!(
        out.status.code(),
        Some(if expected.is_empty() { 0 } else { 1 }),
        "{scenario}: {report}"
    );
    let actual = findings
        .iter()
        .map(|finding| {
            assert_eq!(finding["file"], "src/query.ts", "{scenario}: {report}");
            (
                finding["line"].as_u64().unwrap(),
                finding["target"].as_str().unwrap(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(actual, expected, "{scenario}: {report}");
}

#[test]
fn unbounded_identifier_remaining_shapes_use_the_public_cli() {
    let rule = "postgres-no-unbounded-sql-identifier";
    for (scenario, line, target) in [
        ("fail-conditional-env", 3, "process.env.SCHEMA"),
        ("fail-concat-env", 2, "process.env.SCHEMA"),
        ("fail-alias-chain", 4, "b"),
        ("fail-env-object-alias", 3, "process.env.SCHEMA"),
        ("fail-process-namespace", 3, "process.env.SCHEMA"),
        ("fail-destructured-rows", 3, "rows"),
        ("fail-destructured-row", 4, "schema"),
        ("fail-iterated-rows", 4, "row"),
        ("fail-iterated-row-alias", 4, "row"),
        ("fail-conditional-row-source", 5, "row"),
        ("fail-diamond-alias", 21, "a18"),
        ("fail-await-env-alias", 3, "a"),
        ("fail-env-fallback-right", 4, "a"),
        ("fail-env-destruct-default", 3, "schema"),
        ("fail-row-default", 4, "row"),
        ("fail-row-nullish", 5, "row"),
        ("fail-row-nullish-right", 5, "row"),
    ] {
        exact_locations(rule, scenario, &[(line, target)]);
    }
    for scenario in [
        "pass-conditional-literals",
        "pass-new-bind-value",
        "pass-projected-http-rows",
        "pass-alias-cycle",
        "pass-alias-shadow",
        "pass-finite-alias",
        "pass-process-namespace-shadow",
        "pass-env-alias-process-parameter",
        "pass-stale-mutable-row",
        "pass-dynamic-env-key",
        "pass-diamond-alias",
        "pass-self-alias-cycle",
    ] {
        exact_locations(rule, scenario, &[]);
    }
}

#[test]
fn mutable_registry_remaining_shapes_use_the_public_cli() {
    let rule = "postgres-no-mutable-sql-registry";
    for scenario in [
        "fail-literal-computed",
        "fail-dynamic-key",
        "fail-static-require",
        "fail-many-const-requires",
        "fail-require-namespace",
        "fail-namespace-reexport",
        "fail-namespace-named-reexport",
    ] {
        exact_locations(rule, scenario, &[(3, "src/tables.ts")]);
    }
    for scenario in [
        "pass-dynamic-unrelated",
        "pass-dynamic-require",
        "pass-shadowed-require",
        "pass-require-other-target",
        "pass-namespace-other-target",
        "pass-namespace-cycle",
        "pass-unknown-namespace-key",
        "pass-require-rebound",
    ] {
        exact_locations(rule, scenario, &[]);
    }
}
