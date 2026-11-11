use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn configured(root: &Path) -> serde_yaml::Value {
    let text = std::fs::read_to_string(root.join(".no-mistakes.yml")).unwrap();
    let config: serde_yaml::Value = serde_yaml::from_str(&text).unwrap();
    config["checks"]["commands"]
        .as_sequence()
        .unwrap()
        .iter()
        .find(|check| check["name"] == "rust-architecture-validation")
        .expect("Rust source changes must select architecture validation")
        .clone()
}

#[test]
fn repository_config_keeps_the_fixture_architecture_command() {
    let root = repository_root();
    let fixture = root.join("test-cases/impacted-checks/rust-architecture-validation");
    assert_eq!(configured(&root), configured(&fixture));
}

#[test]
fn source_and_configuration_changes_select_architecture_validation_snapshot() {
    let root = repository_root().join("test-cases/impacted-checks/rust-architecture-validation");
    let mut reports = BTreeMap::new();
    for changed in [
        "crates/no-mistakes/src/check_runner.rs",
        "crates/no-mistakes/src/check_tasks/react.rs",
        "crates/no-mistakes/src/check_runner/tests/architecture.rs",
        "crates/no-mistakes/src/tests/plan/framework.rs",
        "crates/no-mistakes/src/deleted.rs",
        "crates/no-mistakes/Cargo.toml",
        "Cargo.toml",
        "Cargo.lock",
        "rust-toolchain.toml",
        ".no-mistakes.yml",
        "crates/no-mistakes/tests/cli.rs",
        "crates/other/src/lib.rs",
        "docs/cli/impacted-checks.md",
    ] {
        let output = std::process::Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
            .args(["impacted-checks", changed, "--root"])
            .arg(&root)
            .args(["--generic-only", "--format", "json"])
            .output()
            .unwrap();
        assert!(output.status.success(), "{changed}: {output:?}");
        let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(report["warnings"], serde_json::json!([]));
        assert_eq!(report["fallback_triggered"], false);
        reports.insert(changed, report);
    }
    insta::with_settings!({
        snapshot_path => "../common/snapshots",
        prepend_module_to_snapshot => false,
        omit_expression => true,
    }, {
        insta::assert_snapshot!(
            "rust_architecture_validation",
            serde_json::to_string_pretty(&reports).unwrap()
        );
    });
}
