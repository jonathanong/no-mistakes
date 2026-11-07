use super::check_json_snapshot::{assert_saved_fixture, rules_path};

fn snapshot(shape: &str) {
    let root = rules_path("postgres-bounded-statements", "issue-3114-exact", "fixture");
    assert_saved_fixture(
        &format!("postgres_bounded_issue_3114_exact_{shape}"),
        &[(shape, root.clone(), root.join(format!("{shape}.yml")))],
    );
}

#[test]
fn postgres_bounded_issue_3114_exact_r1_check_json_snapshot() {
    snapshot("r1");
}

#[test]
fn postgres_bounded_issue_3114_exact_r2_check_json_snapshot() {
    snapshot("r2");
}

#[test]
fn postgres_bounded_issue_3114_exact_r3_check_json_snapshot() {
    snapshot("r3");
}

#[test]
fn postgres_bounded_issue_3114_exact_r4_check_json_snapshot() {
    snapshot("r4");
}

#[test]
fn postgres_bounded_issue_3114_exact_off_check_json_snapshot() {
    snapshot("off");
}

#[test]
fn postgres_bounded_issue_3114_exact_off_sql_check_json_snapshot() {
    snapshot("off-sql");
}
