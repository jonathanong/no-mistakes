use super::check_json_snapshot::{assert_saved_fixture_report, rules_path};
use no_mistakes::config::v2::{load_v2_config, Effort};
#[path = "common/git_tracked_rule.rs"]
mod git_tracked_rule;

#[test]
fn postgres_lock_inventory_tracked_default_baseline_check_json_snapshot() {
    let fixture = git_tracked_rule::materialize_rule("postgres-lock-inventory", "default-baseline");
    let root = fixture.path();
    let cases = [
        ("tracked", root.to_path_buf(), Default::default()),
        (
            "null-effort",
            root.to_path_buf(),
            root.join("null-effort.yml"),
        ),
        (
            "empty-effort",
            root.to_path_buf(),
            root.join("empty-effort.yml"),
        ),
    ];
    let reports =
        assert_saved_fixture_report("postgres_lock_inventory_default_baseline", &cases, &[], &[]);
    assert_eq!(reports["tracked"]["exitCode"], 0);
    assert_eq!(reports["tracked"]["report"]["rules"], serde_json::json!([]));
    for (label, _, config_path) in &cases[1..] {
        assert_eq!(reports[*label], reports["tracked"]);
        let mut config = load_v2_config(root, Some(config_path)).unwrap();
        config.effort = Some(Effort::Low);
        let requested = config.requested_rule_analysis(&config.rules[0]).unwrap();
        assert_eq!(requested.effort, Effort::Low);
    }
}

#[test]
fn postgres_lock_inventory_check_json_snapshot() {
    let mut cases = [
        "added",
        "clean",
        "unknown",
        "gap-multiplicity",
        "moved",
        "removed",
        "suppressed",
        "helper-only",
        "routine",
        "invalid-baseline",
        "imported-helper",
        "imported-alternatives",
        "callbacks",
    ]
    .map(|name| {
        let root = rules_path("postgres-lock-inventory", "fixture", name);
        (name, root.clone(), root.join(".no-mistakes.yml"))
    })
    .to_vec();
    let root = rules_path("postgres-lock-inventory", "fixture", "gap-multiplicity");
    cases.push(("effort-max", root.clone(), root.join("effort.yml")));
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
    assert_eq!(
        reports["gap-multiplicity"]["report"]["rules"]
            .as_array()
            .unwrap()
            .len(),
        7
    );
    assert_eq!(reports["effort-max"], reports["gap-multiplicity"]);
    let routine = reports["routine"]["report"]["rules"].as_array().unwrap();
    assert_eq!(routine.len(), 4);
    let invalid = reports["invalid-baseline"]["report"]["rules"]
        .as_array()
        .unwrap();
    assert_eq!(invalid.len(), 2);
    assert!(invalid.iter().any(|finding| {
        finding["file"] == "baseline.json" && finding["target"] == "invalid-baseline"
    }));
    assert!(invalid
        .iter()
        .any(|finding| finding["rule"] == "postgres-no-offset"));
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
    let alternatives = reports["imported-alternatives"]["report"]["rules"]
        .as_array()
        .unwrap();
    assert_eq!(alternatives.len(), 2);
    assert!(alternatives.iter().all(|finding| finding["line"] == 5));
    let callbacks = reports["callbacks"]["report"]["rules"].as_array().unwrap();
    assert_eq!(callbacks.len(), 2);
    assert!(callbacks
        .iter()
        .any(|finding| finding["line"] == 5 && finding["target"] == "for-update"));
    assert!(callbacks
        .iter()
        .any(|finding| finding["line"] == 12 && finding["target"] == "unanalyzable"));
    assert_eq!(reports["suppressed"]["exitCode"], 0);
    assert_eq!(reports["moved"]["exitCode"], 0);
    assert_eq!(reports["removed"]["exitCode"], 1);
}

#[test]
fn inventory_invalid_effort_names_rule_and_allowed_values() {
    let root = rules_path("postgres-lock-inventory", "fixture", "gap-multiplicity");
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["--lock-timeout", "0", "check", "--root"])
        .arg(&root)
        .args(["--config", "invalid-effort.yml", "--format", "json"])
        .args(["--effort", "max"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    let message = String::from_utf8(output.stderr).unwrap();
    assert!(
        message.contains("postgres-lock-inventory: invalid options.effort"),
        "{message}"
    );
    assert!(
        message.contains("low") && message.contains("max"),
        "{message}"
    );
}

#[test]
fn inventory_preserves_distinct_gaps_and_rejects_unsupported_analysis_options() {
    let root = rules_path("postgres-lock-inventory", "fixture", "gap-multiplicity");
    let report = no_mistakes::postgres_lock_inventory::run(
        &no_mistakes::postgres_lock_inventory::PostgresLockInventoryOptions {
            root: root.clone(),
            config: Some(".no-mistakes.yml".into()),
            write_baseline: false,
        },
    )
    .unwrap();
    let gaps: Vec<_> = report
        .unanalyzed
        .iter()
        .map(|gap| (gap.file.as_str(), gap.line, gap.reason.as_str()))
        .collect();
    assert_eq!(
        gaps,
        [
            ("app.ts", 3, "executor SQL could not be resolved"),
            ("app.ts", 3, "executor SQL could not be resolved"),
            ("app.ts", 7, "SQL could not be parsed completely"),
            ("imported.ts", 5, "SQL could not be parsed completely"),
            ("imported.ts", 5, "SQL could not be parsed completely"),
            ("locks.sql", 1, "SQL could not be parsed completely"),
            ("locks.sql", 1, "SQL could not be parsed completely"),
        ]
    );
    for (config, key) in [
        ("legacy-policy.yml", "unanalyzableSql"),
        ("shared-policy.yml", "unanalyzable"),
    ] {
        let error =
            no_mistakes::config::v2::load_v2_config(&root, Some(&root.join(config))).unwrap_err();
        assert!(
            error
                .to_string()
                .contains(&format!("does not support options.{key}")),
            "{error:#}"
        );
    }
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
