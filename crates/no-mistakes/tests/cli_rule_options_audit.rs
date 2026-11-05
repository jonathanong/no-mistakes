use super::check_json_snapshot::{assert_saved_fixture, rules_path};
use std::path::PathBuf;
use std::process::{Command, Output};

fn fixture(name: &str) -> PathBuf {
    rules_path("options-audit", "fixture", name)
}

fn config(name: &str) -> PathBuf {
    let root = fixture(name);
    if root.join("config.json").exists() {
        root.join("config.json")
    } else {
        root.join(".no-mistakes.yml")
    }
}

fn check(name: &str, jobs: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["--jobs", jobs, "check", "--root"])
        .arg(fixture(name))
        .arg("--config")
        .arg(config(name))
        .args(["--format", "json"])
        .output()
        .unwrap()
}

#[test]
fn rule_options_json_matches_snapshot() {
    let names = [
        "doc-substring-alone",
        "doc-substring-with-required-file",
        "doc-missing-file",
        "doc-substring-present",
        "doc-substring-filtered",
        "renamed-entry-extensions",
        "renamed-entry-scope",
        "renamed-both-names",
        "renamed-entry-overrides",
        "package-custom-parse",
        "package-advise",
        "package-custom-findings",
        "package-custom-advise",
        "layout-entry-test-options",
        "layout-top-test-options",
        "layout-entry-overrides",
        "version-control",
        "version-blank-anchors",
        "email-html",
        "email-literal",
        "email-percent",
        "email-html-forms",
        "email-fully-encoded",
        "email-empty-domains",
        "finite-shared-yaml",
        "finite-valid-low",
        "timeout-literals-stale-allow",
    ];
    let cases: Vec<_> = names
        .iter()
        .map(|name| (*name, fixture(name), config(name)))
        .collect();
    assert_saved_fixture("rule_options_audit", &cases);
}

#[test]
fn required_options_and_yaml_are_hard_errors() {
    let cases = [
        ("finite-missing-name", "sets[0].name"),
        ("finite-missing-kind-empty", "sets[0].kind"),
        ("finite-missing-comparison-right", "comparisons[0].right"),
        ("finite-missing-left", "comparisons[0].left"),
        ("finite-blank-name", "sets[0].name"),
        ("finite-blank-kind", "sets[0].kind"),
        (
            "finite-invalid-yaml-low",
            "could not parse values.yml as YAML",
        ),
        ("finite-malformed-low", "could not parse values.yml as YAML"),
        ("finite-malformed-max", "could not parse values.yml as YAML"),
        ("version-missing-source-option", "sourceFile"),
        ("version-missing-source-key", "sourceKey"),
        ("version-missing-anchors", "anchors is required"),
        ("version-blank-sourceFile", "sourceFile"),
        ("version-blank-sourceKey", "sourceKey"),
        ("version-missing-options", "sourceFile"),
        ("email-missing-required", "bannedDomains"),
        ("email-missing-options", "bannedDomains"),
        ("email-null-options", "bannedDomains"),
        ("timeout-literals-missing-file", "allow[0].file"),
    ];
    let mut diagnostics = serde_json::Map::new();
    for (name, expected) in cases {
        let one = check(name, "1");
        let two = check(name, "2");
        assert!(!one.status.success(), "{name} unexpectedly passed");
        assert_eq!(one.status.code(), two.status.code(), "{name}");
        let stderr = String::from_utf8(one.stderr).unwrap();
        assert!(stderr.contains(expected), "{name}: {stderr}");
        assert_eq!(stderr, String::from_utf8(two.stderr).unwrap(), "{name}");
        diagnostics.insert(
            name.to_string(),
            serde_json::json!({
                "exitCode": one.status.code(), "stderr": stderr,
            }),
        );
    }
    insta::with_settings!({snapshot_path => "common/snapshots", prepend_module_to_snapshot => false, omit_expression => true}, {
        insta::assert_snapshot!("rule_options_errors", serde_json::to_string_pretty(&diagnostics).unwrap());
    });
}

fn report(name: &str) -> serde_json::Value {
    let output = check(name, "1");
    serde_json::from_slice(&output.stdout).unwrap()
}

#[test]
fn options_change_only_the_declared_policy() {
    for name in [
        "layout-entry-test-options",
        "layout-top-test-options",
        "finite-shared-yaml",
        "finite-valid-low",
        "email-empty-domains",
        "version-blank-anchors",
        "doc-substring-present",
    ] {
        assert_eq!(report(name)["rules"], serde_json::json!([]), "{name}");
    }
    for name in ["doc-substring-alone", "doc-substring-with-required-file"] {
        let report = report(name);
        assert_eq!(report["rules"].as_array().unwrap().len(), 1);
        assert!(report["rules"][0]["message"]
            .as_str()
            .unwrap()
            .contains("missing required substring"));
    }
    let renamed = report("renamed-entry-overrides");
    let files: Vec<_> = renamed["rules"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| f["file"].as_str().unwrap())
        .collect();
    assert_eq!(files, ["backend/legacy.js", "web/middleware.ts"]);
    for (name, channel) in [
        ("package-custom-parse", "rules"),
        ("package-custom-findings", "rules"),
        ("package-custom-advise", "advisories"),
    ] {
        let report = report(name);
        let messages = report[channel].as_array().unwrap();
        assert_eq!(messages.len(), 1);
        assert!(messages[0]["message"]
            .as_str()
            .unwrap()
            .starts_with("AUDIT CUSTOM "));
    }
    let email = report("email-html-forms");
    let findings = email["rules"].as_array().unwrap();
    assert_eq!(findings.len(), 8, "{email}");
    assert!(findings.iter().all(|f| f["target"] == "example.com"));
    assert!(findings
        .iter()
        .any(|f| f["import"] == "person&commat;example&period;com" && f["line"] == 3));
    assert_eq!(
        report("email-fully-encoded")["rules"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
}
