use super::check_json_snapshot::{assert_saved_fixture, assert_saved_fixture_report, rules_path};
use std::path::PathBuf;

fn value_const_root() -> PathBuf {
    let mut path = rules_path("postgres-bounded-statements", "value-const-holes", "src");
    path.pop();
    path
}

#[test]
fn postgres_value_const_holes_check_json_snapshot() {
    assert_saved_fixture(
        "postgres_value_const_holes",
        &[(
            "value_const_holes",
            value_const_root(),
            rules_path(
                "postgres-bounded-statements",
                "configs",
                "value-const-holes.yml",
            ),
        )],
    );
}

#[test]
fn postgres_raw_value_positions_check_json_snapshot() {
    let reports = assert_saved_fixture_report(
        "postgres_raw_value_positions",
        &[(
            "value_positions",
            value_const_root(),
            rules_path("postgres-bounded-statements", "configs", "value-positions.yml"),
        )],
        &[],
        &["--include-sql-verdicts"],
    );
    let report = &reports["value_positions"]["report"];
    let verdicts = report["sqlVerdicts"].as_array().unwrap();
    assert_eq!(verdicts.len(), 36, "{report}");
    assert!(
        verdicts.iter().all(|row| row["status"] != "abstain"),
        "{report}"
    );
    assert!(report["advisories"].as_array().unwrap().is_empty(), "{report}");
    assert!(!report["rules"].as_array().unwrap().is_empty(), "{report}");
}
