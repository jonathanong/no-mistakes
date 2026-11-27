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

#[test]
fn inventory_baseline_unanalyzed_ratchets_both_sites_and_unresolved_inputs() {
    let variants = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres-lock-inventory/baseline-unanalyzed-variants");
    let fixture = saved_fixture::materialize("postgres-lock-inventory", "baseline-unanalyzed");
    let root = fixture.path();

    let initial_read = inventory(root, false);
    assert_eq!(initial_read.status.code(), Some(1));
    let before: Value = serde_json::from_slice(&initial_read.stdout).unwrap();
    assert_eq!(before["baselineUnanalyzed"], true);
    assert_eq!(
        before["unanalyzedByReason"]["executor SQL could not be resolved"],
        3
    );
    assert_eq!(
        before["unanalyzedByReason"]["SQL could not be parsed completely"],
        1
    );
    assert_eq!(before["unanalyzedByDirectory"]["src/orders"], 2);
    assert_eq!(before["unanalyzedByDirectory"]["src/billing"], 1);
    assert_eq!(before["unanalyzedByDirectory"]["migrations"], 1);

    let written = inventory(root, true);
    assert!(
        written.status.success(),
        "{}",
        String::from_utf8_lossy(&written.stderr)
    );
    let wrote: Value = serde_json::from_slice(&written.stdout).unwrap();
    assert_eq!(wrote["baselineUpdated"], true);
    assert_eq!(wrote["added"], serde_json::json!([]));
    assert_eq!(wrote["unanalyzedAdded"], serde_json::json!([]));
    let baseline_v1: Value =
        serde_json::from_slice(&std::fs::read(root.join("baseline.json")).unwrap()).unwrap();
    assert_eq!(baseline_v1["version"], 1);
    assert_eq!(baseline_v1["sites"].as_array().unwrap().len(), 2);
    assert_eq!(baseline_v1["unanalyzed"].as_array().unwrap().len(), 3);

    let clean = inventory(root, false);
    assert!(clean.status.success());

    let orders = root.join("src/orders/repo.ts");
    let baseline_bytes = std::fs::read(root.join("baseline.json")).unwrap();
    for variant in [
        "orders-grew.ts",
        "orders-added-lock.ts",
        "orders-swapped-to-unresolved.ts",
    ] {
        std::fs::copy(variants.join(variant), &orders).unwrap();
        assert_eq!(inventory(root, false).status.code(), Some(1), "{variant}");
        let refused = inventory(root, true);
        assert!(!refused.status.success(), "{variant}");
        assert!(
            String::from_utf8_lossy(&refused.stderr).contains("only go down"),
            "{variant}"
        );
        assert_eq!(
            std::fs::read(root.join("baseline.json")).unwrap(),
            baseline_bytes,
            "{variant}"
        );
    }

    std::fs::copy(variants.join("orders-shrunk.ts"), &orders).unwrap();
    let shrunk_read = inventory(root, false);
    assert_eq!(shrunk_read.status.code(), Some(1));
    let shrunk_json: Value = serde_json::from_slice(&shrunk_read.stdout).unwrap();
    assert_eq!(shrunk_json["unanalyzedRemoved"][0]["baselineCount"], 2);
    assert_eq!(shrunk_json["unanalyzedRemoved"][0]["currentCount"], 1);
    assert!(inventory(root, true).status.success());
    assert!(inventory(root, false).status.success());

    std::fs::copy(variants.join("orders-removed-lock.ts"), &orders).unwrap();
    let removed_read = inventory(root, false);
    assert_eq!(removed_read.status.code(), Some(1));
    let removed_json: Value = serde_json::from_slice(&removed_read.stdout).unwrap();
    assert_eq!(removed_json["removed"].as_array().unwrap().len(), 1);
    assert_eq!(removed_json["unanalyzedAdded"], serde_json::json!([]));
    assert!(inventory(root, true).status.success());
    assert!(inventory(root, false).status.success());

    std::fs::remove_file(root.join("migrations/broken.sql")).unwrap();
    assert_eq!(inventory(root, false).status.code(), Some(1));
    assert!(inventory(root, true).status.success());
    let pruned: Value =
        serde_json::from_slice(&std::fs::read(root.join("baseline.json")).unwrap()).unwrap();
    assert_eq!(pruned["sites"].as_array().unwrap().len(), 1);
    assert_eq!(pruned["unanalyzed"].as_array().unwrap().len(), 2);
    assert!(inventory(root, false).status.success());
}
