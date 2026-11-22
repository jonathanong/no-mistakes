use super::check_json_snapshot::{assert_saved_fixture_report, rules_path};

#[test]
fn postgres_lock_inventory_check_json_snapshot() {
    let cases = [
        "added",
        "clean",
        "unknown",
        "moved",
        "removed",
        "suppressed",
        "helper-only",
        "routine",
    ]
    .map(|name| {
        let root = rules_path("postgres-lock-inventory", "fixture", name);
        (name, root.clone(), root.join(".no-mistakes.yml"))
    });
    let reports = assert_saved_fixture_report("postgres_lock_inventory", &cases, &[], &[]);
    assert_eq!(reports["added"]["exitCode"], 1);
    assert_eq!(
        reports["helper-only"]["report"]["rules"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(reports["clean"]["exitCode"], 0);
    assert_eq!(reports["unknown"]["exitCode"], 1);
    assert_eq!(reports["routine"]["exitCode"], 1);
    assert_eq!(reports["suppressed"]["exitCode"], 0);
    assert_eq!(reports["moved"]["exitCode"], 0);
    assert_eq!(reports["removed"]["exitCode"], 1);
}
