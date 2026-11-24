use super::cli_postgres_snapshot_lock_written_rows::snapshot;
use serde_json::Value;

fn lines(report: &Value, category: &str, target: &str) -> Vec<u64> {
    lines_in_file(report, category, target, None)
}

fn lines_in_file(report: &Value, category: &str, target: &str, file: Option<&str>) -> Vec<u64> {
    let mut lines = report[category]
        .as_array()
        .unwrap()
        .iter()
        .filter(|item| item["target"] == target && file.is_none_or(|file| item["file"] == file))
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
    assert_eq!(report["rules"].as_array().unwrap().len(), 11);
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
            (
                "read_only",
                "share-lock-sequences",
                "share-read-only-fail.yml",
            ),
            ("disabled", "share-lock-sequences", "disabled.yml"),
        ],
    );
    for (case, category) in [("advise", "advisories"), ("fail", "rules")] {
        assert_eq!(
            lines_in_file(
                &reports[case]["report"],
                category,
                "share-lock-then-write",
                Some("src/query.ts")
            ),
            [6, 12, 18, 75, 92, 103, 189, 209]
        );
        assert_eq!(
            lines_in_file(
                &reports[case]["report"],
                category,
                "share-lock-then-write",
                Some("src/lock-targets.ts")
            ),
            [5]
        );
    }
    assert_eq!(
        lines_in_file(
            &reports["advise"]["report"],
            "advisories",
            "share-lock-then-write",
            Some("src/row-sequences.ts")
        ),
        [19, 26, 44]
    );
    assert_eq!(
        lines_in_file(
            &reports["advise"]["report"],
            "advisories",
            "unanalyzable",
            Some("src/row-sequences.ts")
        ),
        [37]
    );
    assert_eq!(
        lines_in_file(
            &reports["advise"]["report"],
            "advisories",
            "unanalyzable",
            Some("src/query.ts")
        ),
        [56, 63, 69, 86, 114, 134, 141, 149, 165, 171, 177, 195, 202]
    );
    assert_eq!(
        lines_in_file(
            &reports["advise"]["report"],
            "advisories",
            "unanalyzable",
            Some("src/variant-sequences.ts")
        ),
        [5, 11]
    );
    assert!(reports["read_only"]["report"]["rules"]
        .as_array()
        .unwrap()
        .is_empty());
    assert!(lines(
        &reports["disabled"]["report"],
        "advisories",
        "share-lock-then-write"
    )
    .is_empty());
}

#[test]
fn postgres_lock_written_rows_share_key_updates_check_json_snapshot() {
    let reports = snapshot(
        "postgres_lock_written_rows_share_key_updates",
        &[
            ("catalog", "key-share-key-writes", "share-key-writes.yml"),
            ("no_catalog", "key-share-key-writes", "share-advise.yml"),
        ],
    );
    for (case, expected) in [("catalog", &[5, 23, 35, 41][..]), ("no_catalog", &[41][..])] {
        let report = &reports[case]["report"];
        assert_eq!(
            lines(report, "advisories", "share-lock-then-write"),
            expected
        );
        assert!(lines(report, "advisories", "unanalyzable").is_empty());
    }
}

#[test]
fn postgres_lock_written_rows_share_uncertainty_check_json_snapshot() {
    let reports = snapshot(
        "postgres_lock_written_rows_share_uncertainty",
        &[("advise", "share-lock-uncertainty", "share-uncertainty.yml")],
    );
    let advise = &reports["advise"]["report"];
    assert!(lines(advise, "advisories", "share-lock-then-write").is_empty());
    assert_eq!(
        lines(advise, "advisories", "unanalyzable"),
        [8, 16, 22, 29, 37]
    );
}

#[test]
fn postgres_lock_written_rows_share_batch_edges_check_json_snapshot() {
    let reports = snapshot(
        "postgres_lock_written_rows_share_batch_edges",
        &[("advise", "share-lock-batch-edges", "share-uncertainty.yml")],
    );
    let advise = &reports["advise"]["report"];
    assert_eq!(
        lines(advise, "advisories", "share-lock-then-write"),
        [7, 20, 30]
    );
    assert_eq!(lines(advise, "advisories", "unanalyzable"), [6, 13, 37]);
}

#[test]
fn postgres_lock_written_rows_redundant_prelock_quoted_check_json_snapshot() {
    let reports = snapshot(
        "postgres_lock_written_rows_redundant_prelock_quoted",
        &[(
            "advise",
            "redundant-prelock-quoted",
            "redundant-no-catalog.yml",
        )],
    );
    let advise = &reports["advise"]["report"];
    assert_eq!(lines(advise, "advisories", "redundant-prelock"), [12]);
    assert!(lines(advise, "advisories", "unanalyzable").is_empty());
}

#[test]
fn postgres_lock_written_rows_redundant_prelock_boundaries_check_json_snapshot() {
    let reports = snapshot(
        "postgres_lock_written_rows_redundant_prelock_boundaries",
        &[(
            "advise",
            "redundant-prelock-boundaries",
            "redundant-advise.yml",
        )],
    );
    let advise = &reports["advise"]["report"];
    assert_eq!(
        lines_in_file(
            advise,
            "advisories",
            "redundant-prelock",
            Some("src/adjacency.ts")
        ),
        [5, 18]
    );
    assert_eq!(
        lines_in_file(
            advise,
            "advisories",
            "redundant-prelock",
            Some("src/later-access.ts")
        ),
        [19, 27]
    );
    assert_eq!(
        lines(advise, "advisories", "redundant-prelock"),
        [5, 18, 19, 27]
    );
    assert_eq!(
        lines_in_file(
            advise,
            "advisories",
            "unanalyzable",
            Some("src/unsupported-sql.ts")
        ),
        [5, 11, 17, 23, 35, 47, 53, 71]
    );
}

#[test]
fn postgres_lock_written_rows_redundant_prelocks_check_json_snapshot() {
    let reports = snapshot(
        "postgres_lock_written_rows_redundant_prelocks",
        &[
            ("advise", "redundant-prelocks", "redundant-advise.yml"),
            (
                "no_catalog",
                "redundant-prelocks",
                "redundant-no-catalog.yml",
            ),
            (
                "invalid_fail",
                "redundant-prelocks",
                "redundant-invalid-fail.yml",
            ),
            ("disabled", "redundant-prelocks", "disabled.yml"),
        ],
    );
    let advise = &reports["advise"]["report"];
    for case in ["advise", "no_catalog"] {
        assert_eq!(
            lines_in_file(
                &reports[case]["report"],
                "advisories",
                "unanalyzable",
                Some("src/inheritance.ts")
            ),
            [5, 11]
        );
        assert!(lines_in_file(
            &reports[case]["report"],
            "advisories",
            "redundant-prelock",
            Some("src/inheritance.ts")
        )
        .is_empty());
    }

    assert_eq!(
        lines_in_file(
            advise,
            "advisories",
            "unanalyzable",
            Some("src/variant-sequences.ts")
        ),
        [5, 11]
    );
    assert_eq!(
        lines_in_file(
            advise,
            "advisories",
            "redundant-prelock",
            Some("src/query.ts")
        ),
        [5, 11, 74, 118]
    );
    assert_eq!(
        lines_in_file(advise, "advisories", "unanalyzable", Some("src/query.ts")),
        [48, 68, 81, 100, 106, 124]
    );
    assert_eq!(
        lines_in_file(
            advise,
            "advisories",
            "redundant-prelock",
            Some("src/stable-key.ts")
        ),
        [5]
    );
    assert_eq!(
        lines_in_file(
            advise,
            "advisories",
            "unanalyzable",
            Some("src/stable-key.ts")
        ),
        [11]
    );
    assert!(lines_in_file(
        &reports["advise"]["report"],
        "advisories",
        "redundant-prelock",
        Some("src/upsert-source-bounds.ts"),
    )
    .is_empty());
    assert!(lines_in_file(
        advise,
        "advisories",
        "redundant-prelock",
        Some("src/unique-key.ts")
    )
    .is_empty());
    assert_eq!(
        lines_in_file(
            advise,
            "advisories",
            "unanalyzable",
            Some("src/unique-key.ts")
        ),
        [5, 11, 17, 24]
    );
    assert_eq!(
        lines(
            &reports["no_catalog"]["report"],
            "advisories",
            "redundant-prelock"
        ),
        [11]
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
