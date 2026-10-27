use super::check_json_snapshot::{assert_saved_fixture, rules_path};
use std::path::PathBuf;

fn test_state_fixture() -> PathBuf {
    let mut path = rules_path("postgres-no-test-state-mutation", "fixture", "placeholder");
    path.pop();
    path
}

#[test]
fn postgres_unmatched_executors_check_json_snapshot() {
    let lock = "postgres-lock-ordering";
    let tests = "postgres-no-test-state-mutation";
    let lock_root = rules_path(lock, "fixture", "fail-unmatched-executor-names");
    let lock_config = |name: &str| rules_path(lock, "configs", name);
    // include is src/lock.ts, or the two test-state paths. Lock ordering on
    // lock.ts reports nothing of its own, so unmatched counts there match
    // rules.length. The production case may also report unanalyzable-sql
    // because tests/dynamic.test.ts is in the file list; the unmatched
    // openTransaction / 1 file(s) finding must still remain. Exact isolated
    // count 1 may not equal rules.length.
    assert_saved_fixture(
        "postgres_unmatched_executors",
        &[
            // No unmatched-executor-name message.
            (
                "unmatched_not_requested",
                lock_root.clone(),
                lock_config("unmatched-not-requested.yml"),
            ),
            // Message contains `openTransacton`, `1 file(s)`, and
            // "a relative import matches only when it resolves into that package".
            (
                "unmatched_typo",
                lock_root.clone(),
                lock_config("unmatched-typo.yml"),
            ),
            // Two findings; some message contains `0 file(s)`.
            (
                "unmatched_excluded",
                lock_root.clone(),
                lock_config("unmatched-excluded.yml"),
            ),
            // Message contains "any module".
            (
                "unmatched_empty_specifier",
                lock_root,
                lock_config("unmatched-empty-specifier.yml"),
            ),
            (
                "unmatched_production_factory",
                test_state_fixture(),
                rules_path(tests, "configs", "unmatched-production.yml"),
            ),
        ],
    );
}
