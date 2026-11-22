use super::cli_postgres_snapshot_lock_written_rows::snapshot;
use serde_json::Value;

fn lines(report: &Value, category: &str, target: &str) -> Vec<u64> {
    let mut lines = report[category]
        .as_array()
        .unwrap()
        .iter()
        .filter(|item| item["target"] == target)
        .map(|item| item["line"].as_u64().unwrap())
        .collect::<Vec<_>>();
    lines.sort_unstable();
    lines
}

#[test]
fn postgres_lock_written_rows_advisory_ban_existing_check_json_snapshot() {
    let reports = snapshot(
        "postgres_lock_written_rows_advisory_ban_existing",
        &[(
            "all_variants_and_path_scope",
            "advisory-ban-existing",
            "advisory-ban.yml",
        )],
    );
    let report = &reports["all_variants_and_path_scope"]["report"];
    assert_eq!(report["rules"].as_array().unwrap().len(), 10);
    assert!(report["rules"]
        .as_array()
        .unwrap()
        .iter()
        .all(|finding| finding["file"] != "src/allowed.ts"));
}

#[test]
fn postgres_lock_written_rows_locking_cte_existing_check_json_snapshot() {
    let reports = snapshot(
        "postgres_lock_written_rows_locking_cte_existing",
        &[(
            "other_relation_and_later_write",
            "locking-cte-existing",
            "locking-cte.yml",
        )],
    );
    assert_eq!(
        lines(
            &reports["other_relation_and_later_write"]["report"],
            "advisories",
            "row-lock-unwritten-relation"
        ),
        [5, 16]
    );
}

#[test]
fn postgres_lock_written_rows_share_lock_sequences_check_json_snapshot() {
    let reports = snapshot(
        "postgres_lock_written_rows_share_lock_sequences",
        &[
            ("advise", "share-lock-sequences", "share-advise.yml"),
            ("fail", "share-lock-sequences", "share-fail.yml"),
            ("disabled", "share-lock-sequences", "disabled.yml"),
        ],
    );
    for (case, category) in [("advise", "advisories"), ("fail", "rules")] {
        assert_eq!(
            lines(&reports[case]["report"], category, "share-lock-then-write"),
            [6, 12, 18, 75, 92, 98, 103]
        );
    }
    assert_eq!(
        lines(&reports["advise"]["report"], "advisories", "unanalyzable"),
        [56, 63, 69, 86, 114, 134, 141, 149, 165]
    );
    assert!(lines(
        &reports["disabled"]["report"],
        "advisories",
        "share-lock-then-write"
    )
    .is_empty());
}

#[test]
fn postgres_lock_written_rows_redundant_prelocks_check_json_snapshot() {
    let reports = snapshot(
        "postgres_lock_written_rows_redundant_prelocks",
        &[
            ("advise", "redundant-prelocks", "redundant-advise.yml"),
            (
                "invalid_fail",
                "redundant-prelocks",
                "redundant-invalid-fail.yml",
            ),
            ("disabled", "redundant-prelocks", "disabled.yml"),
        ],
    );
    assert_eq!(
        lines(
            &reports["advise"]["report"],
            "advisories",
            "redundant-prelock"
        ),
        [5, 11, 74]
    );
    assert_eq!(
        lines(&reports["advise"]["report"], "advisories", "unanalyzable"),
        [48, 68, 81]
    );
    assert!(
        reports["invalid_fail"]["report"]["incomplete"][0]["message"]
            .as_str()
            .unwrap()
            .contains("supports only advise or ignore")
    );
    assert!(lines(
        &reports["disabled"]["report"],
        "advisories",
        "redundant-prelock"
    )
    .is_empty());
}
