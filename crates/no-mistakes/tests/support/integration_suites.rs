//! Keep explicit Cargo test registration complete as integration modules are added.
use std::collections::BTreeMap;
use std::path::Path;

#[path = "integration_registration.rs"]
mod integration_registration;
#[path = "integration_registration_tests.rs"]
mod integration_registration_tests;

#[test]
fn every_integration_source_is_registered_once() {
    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let tests = crate_root.join("tests");
    let manifest: toml::Value =
        toml::from_str(&std::fs::read_to_string(crate_root.join("Cargo.toml")).unwrap()).unwrap();
    assert_eq!(manifest["package"]["autotests"].as_bool(), Some(false));
    let mut sources = Vec::new();
    for target in manifest["test"].as_array().unwrap() {
        // Feature-gating a suite would silently drop formerly unconditional tests.
        assert!(target.get("required-features").is_none());
        let source = crate_root.join(target["path"].as_str().unwrap());
        assert_eq!(source.parent(), Some(tests.as_path()));
        sources.push((
            source,
            target["name"].as_str().unwrap().starts_with("suite_"),
        ));
    }
    let registrations = integration_registration::collect(&tests, &sources).unwrap();
    let expected: BTreeMap<_, _> = std::fs::read_dir(&tests)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter_map(|path| {
            if path.extension().is_some_and(|extension| extension == "rs") {
                Some(path)
            } else if path.is_dir() && path.join("main.rs").is_file() {
                // Cargo also auto-discovered tests/<name>/main.rs before consolidation.
                Some(path.join("main.rs"))
            } else {
                None
            }
        })
        .map(|path| {
            (
                path.strip_prefix(&tests)
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .replace('\\', "/"),
                1,
            )
        })
        .collect();
    // New root sources must join a suite; duplicates and missing files fail too.
    assert_eq!(registrations, expected);
}

#[test]
fn changed_suite_modules_select_the_configured_workspace_test_command() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/impacted-checks/rust-integration-validation");
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args([
            "impacted-checks",
            "crates/no-mistakes/tests/cli_test_plan_dotnet.rs",
            "crates/no-mistakes/tests/postgres_catalog_real/bounded.rs",
            // This production directory is named tests but holds planner implementation.
            "crates/no-mistakes/src/tests/plan/framework.rs",
            ".no-mistakes.yml",
            "Cargo.toml",
            "Cargo.lock",
            "crates/no-mistakes/Cargo.toml",
            "--root",
        ])
        .arg(repo_root)
        .args(["--format", "json"])
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let check = report["checks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|check| check["name"] == "rust-integration-validation")
        .expect("changed suite modules must select executable Cargo validation");
    // Filename-derived --test stems no longer identify these integration modules.
    assert_eq!(
        check["command"],
        serde_json::json!([
            "cargo",
            "test",
            "--locked",
            "--workspace",
            "--all-features",
            "--lib",
            "--bins",
            "--tests"
        ])
    );
    assert_eq!(
        check["files"],
        serde_json::json!([
            ".no-mistakes.yml",
            "Cargo.lock",
            "Cargo.toml",
            "crates/no-mistakes/Cargo.toml",
            "crates/no-mistakes/tests/cli_test_plan_dotnet.rs",
            "crates/no-mistakes/tests/postgres_catalog_real/bounded.rs"
        ])
    );
}

#[test]
fn repository_config_keeps_the_workspace_integration_command() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let configured = |root: &Path| -> serde_yaml::Value {
        let text = std::fs::read_to_string(root.join(".no-mistakes.yml")).unwrap();
        let config: serde_yaml::Value = serde_yaml::from_str(&text).unwrap();
        config["checks"]["commands"]
            .as_sequence()
            .unwrap()
            .iter()
            .find(|check| check["name"] == "rust-integration-validation")
            .expect("rust-integration-validation must stay configured")
            .clone()
    };
    let fixture = repo_root.join("test-cases/impacted-checks/rust-integration-validation");
    assert_eq!(configured(&repo_root), configured(&fixture));
}
