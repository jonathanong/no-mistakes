use super::check_json_snapshot::{assert_saved_fixture, rules_path};
use std::path::PathBuf;

fn repo_rel(rel: &str) -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(rel),
    )
}

#[test]
fn postgres_bounded_statements_issue_2961_snapshot() {
    assert_saved_fixture(
        "postgres_bounded_statements_issue_2961",
        &[(
            "issue_2961",
            repo_rel("test-cases/rules/postgres-bounded-statements/fixture"),
            rules_path(
                "postgres-bounded-statements",
                "configs",
                "issue-2961-bounded-false-positives.yml",
            ),
        )],
    );
}

#[test]
fn postgres_bounded_statements_issue_2988_snapshot() {
    assert_saved_fixture(
        "postgres_bounded_statements_issue_2988",
        &[(
            "issue_2988",
            repo_rel("test-cases/rules/postgres-bounded-statements/fixture"),
            rules_path("postgres-bounded-statements", "configs", "issue-2988.yml"),
        )],
    );
}

#[test]
fn postgres_bounded_statements_issue_2961_remainder_snapshot() {
    assert_saved_fixture(
        "postgres_bounded_statements_issue_2961_remainder",
        &[(
            "issue_2961_remainder",
            repo_rel("test-cases/rules/postgres-bounded-statements/fixture"),
            rules_path(
                "postgres-bounded-statements",
                "configs",
                "issue-2961-remainder.yml",
            ),
        )],
    );
}

#[test]
fn postgres_bounded_statements_issue_3100_snapshot() {
    assert_saved_fixture(
        "postgres_bounded_statements_issue_3100",
        &[(
            "issue_3100",
            repo_rel("test-cases/rules/postgres-bounded-statements/issue-3100/fixture"),
            rules_path("postgres-bounded-statements", "configs", "issue-3100.yml"),
        )],
    );
}

#[test]
fn issue_3116_recursive_check_json_snapshot() {
    let root = repo_rel("test-cases/rules/postgres-bounded-statements/issue-3116/fixture");
    assert_saved_fixture(
        "postgres_bounded_issue_3116_recursive",
        &[("recursive", root.clone(), root.join("recursive.yml"))],
    );
}

#[test]
fn issue_3116_correlated_check_json_snapshot() {
    let root = repo_rel("test-cases/rules/postgres-bounded-statements/issue-3116/fixture");
    assert_saved_fixture(
        "postgres_bounded_issue_3116_correlated",
        &[
            ("correlated", root.clone(), root.join("correlated.yml")),
            ("embedded", root.clone(), root.join("embedded.yml")),
            ("lateral_bare", root.clone(), root.join("lateral-bare.yml")),
            (
                "unknown_bare",
                root.clone(),
                root.join("correlated-unknown.yml"),
            ),
        ],
    );
}

#[test]
fn issue_3116_off_check_json_snapshot() {
    let root = repo_rel("test-cases/rules/postgres-bounded-statements/issue-3116/fixture");
    assert_saved_fixture(
        "postgres_bounded_issue_3116_off",
        &[("off", root.clone(), root.join("off.yml"))],
    );
}

#[test]
fn issue_3151_check_json_snapshot() {
    let root = repo_rel("test-cases/rules/postgres-bounded-statements/issue-3151/fixture");
    let cases = [
        "advise",
        "report",
        "ignore",
        "off",
        "dml-update",
        "dml-delete",
    ]
    .map(|name| (name, root.clone(), root.join(format!("{name}.yml"))));
    assert_saved_fixture("postgres_bounded_issue_3151", &cases);
}

#[test]
fn issue_3151_retains_unbounded_controls_without_scalar_pool_advice() {
    let root = repo_rel("test-cases/rules/postgres-bounded-statements/issue-3151/fixture");
    for policy in ["advise", "report", "ignore"] {
        let output = std::process::Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
            .args(["check", "--root"])
            .arg(&root)
            .arg("--config")
            .arg(root.join(format!("{policy}.yml")))
            .args(["--format", "json"])
            .output()
            .unwrap();
        let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(output.status.code(), Some(1), "{report}");
        let findings = report["rules"].as_array().unwrap();
        for file in [
            "different-parameters.sql",
            "different-literals.sql",
            "open-range.sql",
            "or-range.sql",
            "cross-or-control.sql",
            "cross-column-control.sql",
            "cast-mismatch.sql",
            "volatile-range.sql",
            "scalar-different-parameters.sql",
            "unbounded-scalar.sql",
            "unbounded-coalesce.sql",
        ] {
            assert!(
                findings.iter().any(|finding| finding["file"] == file),
                "{policy}: {file}: {report}"
            );
        }
        for file in [
            "closed-parameter.sql",
            "closed-literal.sql",
            "closed-or-branches.sql",
            "scalar-select.sql",
            "scalar-coalesce.sql",
            "scalar-closed.sql",
            "scalar-correlated.sql",
        ] {
            assert!(
                !findings.iter().any(|finding| finding["file"] == file),
                "{policy}: {file}: {report}"
            );
            assert!(
                !report["advisories"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|finding| finding["file"] == file),
                "{policy}: {file}: {report}"
            );
        }
    }
}

#[test]
fn issue_3151_scalar_scopes_check_json_snapshot() {
    let root = repo_rel("test-cases/rules/postgres-bounded-statements/issue-3151-scopes/fixture");
    let cases = ["advise", "report", "ignore", "off"]
        .map(|name| (name, root.clone(), root.join(format!("{name}.yml"))));
    assert_saved_fixture("postgres_bounded_issue_3151_scopes", &cases);
}

#[test]
fn issue_3151_scalar_scope_controls_preserve_only_unbounded_reads() {
    let root = repo_rel("test-cases/rules/postgres-bounded-statements/issue-3151-scopes/fixture");
    for policy in ["advise", "report", "ignore"] {
        let output = std::process::Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
            .args(["check", "--root"])
            .arg(&root)
            .arg("--config")
            .arg(root.join(format!("{policy}.yml")))
            .args(["--format", "json"])
            .output()
            .unwrap();
        let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(output.status.code(), Some(1), "{report}");
        let findings = report["rules"].as_array().unwrap();
        let mut files: Vec<_> = findings
            .iter()
            .filter(|row| row["target"] != "materialized-body")
            .map(|row| row["file"].as_str().unwrap())
            .collect();
        files.sort_unstable();
        files.dedup();
        assert_eq!(
            files,
            [
                "correlated-local-alias-control.sql",
                "correlated-nonunique-control.sql",
                "correlated-or-control.sql",
                "correlated-volatile-control.sql",
                "lateral-local-self-control.sql",
                "lateral-missing-composite-half.sql",
                "lateral-nonunique-control.sql",
                "lateral-shadowed-enclosing-alias.sql",
                "lateral-unbounded-driver.sql",
                "nonlateral-sibling-control.sql",
                "partial-missing-predicate.sql",
                "partial-mixed-case-control.sql",
                "partial-mixed-key-control.sql",
                "partial-wrong-predicate.sql",
                "temporary-inner-control.sql",
                "temporary-outer-control.sql",
                "unbounded-lateral.sql",
                "unbounded-scalar.sql",
                "unbounded-shadowed-column.sql",
                "uncapped-cte-control.sql",
                "unknown-inner-control.sql",
                "unknown-outer-control.sql",
            ],
            "{policy}: {report}"
        );
        assert!(
            report["advisories"]
                .as_array()
                .unwrap()
                .iter()
                .all(|row| matches!(
                    row["file"].as_str().unwrap(),
                    "lateral-local-self-control.sql"
                        | "lateral-missing-composite-half.sql"
                        | "lateral-nonunique-control.sql"
                        | "lateral-shadowed-enclosing-alias.sql"
                        | "nonlateral-sibling-control.sql"
                )),
            "{report}"
        );
    }
}

#[test]
fn issue_3151_embedded_closed_range_check_json_snapshot() {
    let root = repo_rel("test-cases/rules/postgres-bounded-statements/issue-3151-embedded/fixture");
    let cases = ["advise", "report", "ignore", "off"]
        .map(|name| (name, root.clone(), root.join(format!("{name}.yml"))));
    assert_saved_fixture("postgres_bounded_issue_3151_embedded", &cases);
}

#[test]
fn issue_3151_embedded_ranges_require_the_same_stable_value() {
    let root = repo_rel("test-cases/rules/postgres-bounded-statements/issue-3151-embedded/fixture");
    for policy in ["advise", "report", "ignore"] {
        let output = std::process::Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
            .args(["check", "--root"])
            .arg(&root)
            .arg("--config")
            .arg(root.join(format!("{policy}.yml")))
            .args(["--format", "json"])
            .output()
            .unwrap();
        let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(output.status.code(), Some(1), "{report}");
        let mut files: Vec<_> = report["rules"]
            .as_array()
            .unwrap()
            .iter()
            .map(|row| row["file"].as_str().unwrap())
            .collect();
        files.sort_unstable();
        assert_eq!(
            files,
            [
                "distinct-bounds.ts",
                "intervening-mutation.ts",
                "mutable-bounds.ts",
                "object-binding.ts",
                "unbounded-scalar.ts",
                "volatile-bounds.ts"
            ],
            "{policy}: {report}"
        );
        assert!(
            report["advisories"].as_array().unwrap().is_empty(),
            "{report}"
        );
    }
}

#[test]
fn issue_3151_real_templates_check_json_snapshot() {
    let root = repo_rel("test-cases/rules/postgres-bounded-statements/issue-3151-real/fixture");
    let cases = [
        "embedded",
        "embedded-advise",
        "embedded-ignore",
        "embedded-off",
        "raw",
        "member-advise",
        "member-report",
        "member-ignore",
        "member-off",
    ]
    .map(|name| (name, root.clone(), root.join(format!("{name}.yml"))));
    assert_saved_fixture("postgres_bounded_issue_3151_real_templates", &cases);
}

#[test]
fn issue_3151_member_contract_keeps_unsafe_controls() {
    let root = repo_rel("test-cases/rules/postgres-bounded-statements/issue-3151-real/fixture");
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["check", "--root"])
        .arg(&root)
        .arg("--config")
        .arg(root.join("member-ignore.yml"))
        .args(["--format", "json"])
        .output()
        .unwrap();
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let mut files: Vec<_> = report["rules"]
        .as_array()
        .unwrap()
        .iter()
        .map(|finding| finding["file"].as_str().unwrap())
        .collect();
    files.sort_unstable();
    assert_eq!(
        files,
        [
            "member-accessor-declaration-control.ts",
            "member-alias-control.ts",
            "member-computed-control.ts",
            "member-constructor-control.ts",
            "member-direct-mutation-control.ts",
            "member-distinct-control.ts",
            "member-external-call-control.ts",
            "member-external-tag-control.ts",
            "member-getter-control.ts",
            "member-mutation-control.ts",
            "member-own-data-escape-control.ts",
            "member-proxy-control.ts",
            "member-volatile-control.ts",
            "member-wrapped-mutation-control.ts",
        ],
        "{report}"
    );
}
