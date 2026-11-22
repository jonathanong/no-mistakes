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
        "imported-helper",
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
    assert_eq!(
        reports["imported-helper"]["report"]["rules"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(reports["imported-helper"]["report"]["rules"][0]["line"], 8);
    assert!(reports["imported-helper"]["report"]["advisories"]
        .as_array()
        .unwrap()
        .is_empty());
    assert_eq!(reports["suppressed"]["exitCode"], 0);
    assert_eq!(reports["moved"]["exitCode"], 0);
    assert_eq!(reports["removed"]["exitCode"], 1);
}

#[test]
fn inventory_preserves_existing_rule_locations_check_json_snapshot() {
    let root = rules_path("postgres-lock-inventory", "fixture", "imported-helper");
    let cases = [
        (
            "without_inventory",
            root.clone(),
            root.join("no-offset.yml"),
        ),
        ("with_inventory", root.clone(), root.join("co-demand.yml")),
    ];
    let reports =
        assert_saved_fixture_report("postgres_lock_inventory_shared_locations", &cases, &[], &[]);
    let existing = |name: &str| {
        reports[name]["report"]["rules"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|finding| finding["rule"] == "postgres-no-offset")
            .cloned()
            .collect::<Vec<_>>()
    };
    let before = existing("without_inventory");
    assert!(!before.is_empty());
    assert_eq!(before, existing("with_inventory"));
}
