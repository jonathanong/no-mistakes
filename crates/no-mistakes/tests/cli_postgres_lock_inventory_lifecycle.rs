use super::check_json_snapshot::rules_path;
#[path = "common/saved_fixture.rs"]
mod saved_fixture;
use serde_json::Value;
use std::path::Path;
use std::process::Command;

fn inventory(root: &Path, write: bool) -> std::process::Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_no-mistakes"));
    command
        .args(["postgres", "lock-inventory", "--root"])
        .arg(root)
        .args(["--config", ".no-mistakes.yml", "--lock-timeout", "0"]);
    if write {
        command.arg("--write-baseline");
    }
    command.output().unwrap()
}

#[test]
fn inventory_baseline_writer_only_tightens_and_never_accepts_unanalyzed_sql() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres-lock-inventory/lifecycle");
    let fixture = saved_fixture::materialize("postgres-lock-inventory", "lifecycle");
    let output = inventory(fixture.path(), true);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let initial: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(initial["baselineUpdated"], true);
    assert_eq!(initial["counts"]["for-update"], 2);
    assert_eq!(initial["counts"]["advisory-lock"], 2);
    assert_eq!(initial["counts"]["helper"], 1);
    assert!(inventory(fixture.path(), false).status.success());
    std::fs::remove_file(fixture.path().join("locks.sql")).unwrap();
    assert!(!inventory(fixture.path(), false).status.success());
    assert!(inventory(fixture.path(), true).status.success());
    let shrunk = std::fs::read(fixture.path().join("baseline.json")).unwrap();
    std::fs::copy(root.join("locks.sql"), fixture.path().join("locks.sql")).unwrap();
    let rejected = inventory(fixture.path(), true);
    assert!(!rejected.status.success());
    assert!(String::from_utf8_lossy(&rejected.stderr).contains("only go down"));
    assert_eq!(
        std::fs::read(fixture.path().join("baseline.json")).unwrap(),
        shrunk
    );
    for name in ["unknown", "routine"] {
        let opaque = saved_fixture::materialize_path(&rules_path(
            "postgres-lock-inventory",
            "fixture",
            name,
        ));
        let rejected = inventory(opaque.path(), true);
        assert!(!rejected.status.success());
        assert!(String::from_utf8_lossy(&rejected.stderr).contains("unanalyzed"));
        assert!(!opaque.path().join("baseline.json").exists());
    }
}

#[test]
fn lock_inventory_cli_matches_public_library() {
    let root = rules_path("postgres-lock-inventory", "fixture", "added");
    let output = inventory(&root, false);
    assert_eq!(output.status.code(), Some(1));
    let cli: Value = serde_json::from_slice(&output.stdout).unwrap();
    let library = no_mistakes::postgres_lock_inventory::run(
        &no_mistakes::postgres_lock_inventory::PostgresLockInventoryOptions {
            root,
            config: Some(".no-mistakes.yml".into()),
            write_baseline: false,
        },
    )
    .unwrap();
    assert_eq!(cli, serde_json::to_value(library).unwrap());
}
