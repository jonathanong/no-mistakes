use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_no-mistakes"))
}

fn fixture(scenario: &str) -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-require-query-annotation/fixture")
            .join(scenario),
    )
}

fn config(rule: &str, name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules")
        .join(rule)
        .join("config")
        .join(name)
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

const RULE: &str = "postgres-require-query-annotation";
const ANNOTATION: &str = "postgres_require_query_annotation_fails_without_comment.yml";

#[test]
fn postgres_require_query_annotation_fails_without_comment() {
    let root = fixture("fail");
    let out = check(&root, &config(RULE, ANNOTATION));
    let body = stdout(&out);
    assert!(!out.status.success(), "expected exit 1: {body}");
    assert!(body.contains(RULE), "{body}");
    assert!(body.contains("query.ts"), "{body}");
    assert!(body.contains("/* name */"), "{body}");
}

#[test]
fn postgres_require_query_annotation_fails_for_line_comment() {
    let root = fixture("fail-line-comment");
    let out = check(&root, &config(RULE, ANNOTATION));
    let body = stdout(&out);
    assert!(!out.status.success(), "expected exit 1: {body}");
    assert!(body.contains(RULE), "{body}");
}

#[test]
fn postgres_require_query_annotation_passes_with_block_comment() {
    let root = fixture("pass-annotated");
    let out = check(&root, &config(RULE, ANNOTATION));
    assert!(out.status.success(), "exit non-zero: {}", stdout(&out));
}

#[test]
fn postgres_require_query_annotation_passes_transaction_commands() {
    let root = fixture("pass-transaction");
    let out = check(&root, &config(RULE, ANNOTATION));
    assert!(out.status.success(), "exit non-zero: {}", stdout(&out));
}

#[test]
fn postgres_require_query_annotation_json_has_rule_id() {
    let root = fixture("fail");
    let out = Command::new(bin())
        .args(["check", "--root"])
        .arg(&root)
        .arg("--config")
        .arg(root.join(".no-mistakes.yml"))
        .args(["--format", "json"])
        .output()
        .unwrap();
    let body = stdout(&out);
    assert!(body.contains(RULE), "{body}");
    assert!(!out.status.success());
}

#[test]
fn postgres_require_query_annotation_filesystem_runner_discovers_files() {
    let root = fixture("fail");
    let findings = no_mistakes::codebase::rules::run_filesystem_rules(&root, None).unwrap();
    let body = format!("{findings:?}");
    assert!(!findings.is_empty(), "expected findings");
    assert!(
        findings.iter().any(|finding| {
            finding.rule == no_mistakes::codebase::rules::POSTGRES_REQUIRE_QUERY_ANNOTATION
        }),
        "{body}"
    );
}

#[test]
fn postgres_annotation_helpers_keep_importer_ownership_and_explicit_overrides() {
    let root = fixture("helper-tracing-monorepo");
    for forced in [false, true] {
        let mut command = Command::new(bin());
        command.args(["check", "--root"]).arg(&root);
        command.args(["--format", "json"]);
        if forced {
            command.arg("--tsconfig").arg(root.join("tsconfig.json"));
        }
        let output = command.output().unwrap();
        let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        let rules = report["rules"].as_array().unwrap();
        assert_eq!(output.status.success(), forced, "{report}");
        if forced {
            assert!(rules.is_empty(), "{report}");
        } else {
            assert_eq!(rules.len(), 1, "{report}");
            assert_eq!(rules[0]["file"], "packages/bad/src/query.mts");
            assert_eq!(rules[0]["line"], 3);
            assert_eq!(rules[0]["target"], "annotation");
        }
    }
    let findings = no_mistakes::codebase::rules::run_filesystem_rules(&root, None).unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].file, "packages/bad/src/query.mts");
}

#[test]
fn delayed_try_receivers_share_low_max_proof_and_keep_policy_controlled_abstentions() {
    let root = fixture("delayed-try-receivers");
    for effort in ["low", "max"] {
        for policy in ["report", "advise", "ignore"] {
            let output = Command::new(bin())
                .args(["check", "--root"])
                .arg(&root)
                .arg("--config")
                .arg(root.join(format!("{effort}-{policy}.yml")))
                .args(["--format", "json", "--include-sql-verdicts"])
                .output()
                .unwrap();
            let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
            assert_eq!(output.status.code(), Some(1), "{effort}/{policy}: {report}");
            let findings = report["rules"].as_array().unwrap();
            let annotations = findings
                .iter()
                .filter(|row| row["target"] == "annotation")
                .collect::<Vec<_>>();
            assert_eq!(annotations.len(), 1, "{effort}/{policy}: {report}");
            assert_eq!(annotations[0]["file"], "positive.mts");
            assert_eq!(annotations[0]["line"], 9);
            assert!(annotations[0]["message"]
                .as_str()
                .unwrap()
                .contains("/* name */"));
            let unresolved = findings
                .iter()
                .filter(|row| row["target"] == "unresolved-configured-executor-receiver")
                .count();
            assert_eq!(
                unresolved,
                if policy == "report" { 9 } else { 0 },
                "{report}"
            );
            let verdicts = report["sqlVerdicts"].as_array().unwrap();
            let abstentions = verdicts
                .iter()
                .filter(|row| row["subcheck"] == "unresolved-configured-executor-receiver")
                .collect::<Vec<_>>();
            assert_eq!(abstentions.len(), 9, "{report}");
            assert!(
                abstentions.iter().all(
                    |row| row["status"] == "abstain" && row["advisory"] == (policy == "advise")
                ),
                "{report}"
            );
        }
    }
}
