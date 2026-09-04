use std::path::PathBuf;
use std::process::{Command, Output};

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_no-mistakes"))
}

fn stdout(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

fn finite_set_fixture(name: &str) -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/rules/finite-set-consistency")
            .join(name),
    )
}

#[test]
fn finite_set_path_regex_min_size_fails_closed_for_empty_captures() {
    let root = finite_set_fixture("path-regex-empty-minsize");
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

    assert!(!out.status.success(), "{body}");
    assert_eq!(
        rules
            .iter()
            .filter(|finding| finding["rule"] == "finite-set-consistency")
            .count(),
        2,
        "{body}"
    );
    assert!(rules.iter().any(|finding| {
        finding["rule"] == "finite-set-consistency"
            && finding["message"].as_str().is_some_and(|message| {
                message.contains("missingFiles") && message.contains("minSize is 1")
            })
    }));
    assert!(rules.iter().any(|finding| {
        finding["rule"] == "finite-set-consistency"
            && finding["message"].as_str().is_some_and(|message| {
                message.contains("missingLinks") && message.contains("minSize is 1")
            })
    }));
    assert!(rules.iter().all(|finding| {
        finding["rule"] != "finite-set-consistency"
            || !finding["message"]
                .as_str()
                .is_some_and(|message| message.contains("contains"))
    }));
}

#[test]
fn finite_set_yaml_string_selector_passes_and_reports_missing_values() {
    let pass_root = finite_set_fixture("yaml-string-selector/pass");
    let pass = Command::new(bin())
        .args(["check", "--root"])
        .arg(&pass_root)
        .arg("--config")
        .arg(pass_root.join(".no-mistakes.yml"))
        .args(["--format", "json"])
        .output()
        .unwrap();
    assert!(pass.status.success(), "{}", stdout(&pass));

    let fail_root = finite_set_fixture("yaml-string-selector/fail");
    let fail = Command::new(bin())
        .args(["check", "--root"])
        .arg(&fail_root)
        .arg("--config")
        .arg(fail_root.join(".no-mistakes.yml"))
        .args(["--format", "json"])
        .output()
        .unwrap();
    let body = stdout(&fail);
    let json: serde_json::Value = serde_json::from_str(&body).expect("stdout should be json");

    assert!(!fail.status.success(), "{body}");
    assert!(json["rules"].as_array().unwrap().iter().any(|finding| {
        finding["rule"] == "finite-set-consistency"
            && finding["target"] == "@acme/web"
            && finding["message"]
                .as_str()
                .is_some_and(|message| message.contains("permanentPackages contains"))
    }));
}

#[test]
fn finite_set_prefix_transform_filters_a_pnpm_workspace_yaml_and_a_json_package_json() {
    let pass_root = finite_set_fixture("prefix-transform/pass");
    let pass = Command::new(bin())
        .args(["check", "--root"])
        .arg(&pass_root)
        .arg("--config")
        .arg(pass_root.join(".no-mistakes.yml"))
        .args(["--format", "json"])
        .output()
        .unwrap();
    assert!(pass.status.success(), "{}", stdout(&pass));

    let fail_root = finite_set_fixture("prefix-transform/fail");
    let fail = Command::new(bin())
        .args(["check", "--root"])
        .arg(&fail_root)
        .arg("--config")
        .arg(fail_root.join(".no-mistakes.yml"))
        .args(["--format", "json"])
        .output()
        .unwrap();
    let body = stdout(&fail);
    let json: serde_json::Value = serde_json::from_str(&body).expect("stdout should be json");

    assert!(!fail.status.success(), "{body}");
    let rules = json["rules"].as_array().unwrap();
    assert!(rules.iter().any(|finding| {
        finding["rule"] == "finite-set-consistency"
            && finding["target"] == "api"
            && finding["message"]
                .as_str()
                .is_some_and(|message| message.contains("workspaceYamlBackendPackages contains"))
    }));
    assert!(rules.iter().any(|finding| {
        finding["rule"] == "finite-set-consistency"
            && finding["target"] == "cli"
            && finding["message"]
                .as_str()
                .is_some_and(|message| message.contains("packageJsonWorkspaces contains"))
    }));
}
