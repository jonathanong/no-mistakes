use super::check_json_snapshot::{assert_saved_fixture, rules_path};
use serde_json::Value;
use std::process::Command;

fn check(case: &str) -> Value {
    let root = rules_path("postgres-site-identity", "fixture", ".");
    let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["--jobs", "1", "check", "--root"])
        .arg(&root)
        .arg("--config")
        .arg(root.join(format!("{case}.yml")))
        .args(["--format", "json"])
        .output()
        .unwrap();
    serde_json::from_slice(&output.stdout).unwrap()
}

fn snapshot(name: &str, cases: &[&str]) {
    let root = rules_path("postgres-site-identity", "fixture", ".");
    let cases: Vec<_> = cases
        .iter()
        .map(|&case| (case, root.clone(), root.join(format!("{case}.yml"))))
        .collect();
    assert_saved_fixture(name, &cases);
}

#[test]
fn bindings_site_identity_check_json_snapshot() {
    for case in ["bindings-low", "bindings-max"] {
        let report = check(case);
        let findings = report["rules"].as_array().unwrap();
        assert_eq!(findings.len(), 1, "{report}");
        assert_eq!(findings[0]["target"], "accounts.id, accounts.token");
    }
    snapshot(
        "postgres_site_identity_bindings",
        &["bindings-low", "bindings-max", "control-low", "control-max"],
    );
}

#[test]
fn parameterized_cast_site_identity_check_json_snapshot() {
    for case in [
        "casts-low",
        "casts-max",
        "casts-multiword-low",
        "casts-multiword-max",
    ] {
        let report = check(case);
        assert_eq!(report["rules"].as_array().unwrap().len(), 4, "{report}");
    }
    snapshot(
        "postgres_site_identity_casts",
        &[
            "casts-low",
            "casts-max",
            "casts-multiword-low",
            "casts-multiword-max",
            "casts-control-low",
            "casts-control-max",
        ],
    );
}

#[test]
fn optional_invocation_site_identity_check_json_snapshot() {
    for case in [
        "optional-max",
        "optional-placeholders-max",
        "optional-nested-max",
        "optional-imported-max",
        "optional-parameter-max",
    ] {
        let report = check(case);
        let findings = report["rules"].as_array().unwrap();
        let targets: Vec<_> = findings
            .iter()
            .map(|finding| finding["target"].as_str().unwrap())
            .collect();
        assert_eq!(targets.len(), 2, "{report}");
        assert!(
            targets.contains(&"table:accounts, table:orders"),
            "{report}"
        );
        assert!(targets.contains(&"table:accounts, table:other"), "{report}");
    }
    let control = check("optional-alternatives-max");
    assert_eq!(control["rules"].as_array().unwrap().len(), 1, "{control}");
    assert_eq!(
        control["rules"][0]["target"],
        "table:accounts, table:orders, table:other"
    );
    snapshot(
        "postgres_site_identity_optional",
        &[
            "optional-max",
            "optional-placeholders-max",
            "optional-nested-max",
            "optional-imported-max",
            "optional-parameter-max",
            "optional-control-max",
            "optional-alternatives-max",
            "off",
        ],
    );
}

#[test]
fn repeated_expression_site_identity_check_json_snapshot() {
    for case in [
        "repeated-low",
        "repeated-max",
        "repeated-binding-max",
        "repeated-binds-max",
        "repeated-optional-low",
        "repeated-optional-max",
    ] {
        let report = check(case);
        let findings = report["rules"].as_array().unwrap();
        assert_eq!(findings.len(), 2, "{report}");
        assert_ne!(findings[0]["message"], findings[1]["message"]);
        if case.starts_with("repeated-optional") {
            let targets: Vec<_> = findings
                .iter()
                .map(|finding| finding["target"].as_str().unwrap())
                .collect();
            assert!(targets.contains(&"accounts.id"), "{report}");
            assert!(targets.contains(&"accounts.token"), "{report}");
        }
    }
    for case in ["repeated-alternatives-low", "repeated-alternatives-max"] {
        let report = check(case);
        assert_eq!(report["rules"].as_array().unwrap().len(), 1, "{report}");
        assert_eq!(report["rules"][0]["target"], "accounts.id, accounts.token");
    }
    snapshot(
        "postgres_site_identity_repeated",
        &[
            "repeated-low",
            "repeated-max",
            "repeated-binding-low",
            "repeated-binding-max",
            "repeated-binds-low",
            "repeated-binds-max",
            "repeated-optional-low",
            "repeated-optional-max",
            "repeated-control-low",
            "repeated-control-max",
            "repeated-alternatives-low",
            "repeated-alternatives-max",
        ],
    );
}

#[test]
fn imported_suppression_site_identity_check_json_snapshot() {
    let report = check("imported-max");
    let findings = report["rules"].as_array().unwrap();
    assert_eq!(findings.len(), 1, "{report}");
    assert_eq!(findings[0]["file"], "imported.ts");
    assert_eq!(findings[0]["line"], 3);
    assert!(check("suppressed-import-max")["rules"]
        .as_array()
        .unwrap()
        .is_empty());
    snapshot(
        "postgres_site_identity_suppression",
        &["imported-max", "suppressed-import-max", "off"],
    );
}
