use super::check_json_snapshot::{assert_saved_fixture, rules_path};

#[test]
fn postgres_closed_consumer_candidates_check_json_snapshot() {
    let rule = "postgres-require-query-annotation";
    let root = rules_path(rule, "fixture", "closed-consumer-candidates");
    // A named head is not a proof of complete SQL. Both reports retain their
    // independent uncertainty, including mutable/prototype selector controls.
    assert_saved_fixture(
        "postgres_closed_consumer_candidates",
        &[
            (
                "finite_conditional_heads_low",
                rules_path(rule, "fixture", "low-heads"),
                rules_path(rule, "configs", "finite-conditional-consumer-low.yml"),
            ),
            (
                "annotation_low",
                root.clone(),
                rules_path(rule, "configs", "closed-consumer-candidates-low.yml"),
            ),
            (
                "complete_sql_low",
                root,
                rules_path(
                    "postgres-no-test-state-mutation",
                    "configs",
                    "closed-consumer-candidates-sql.yml",
                ),
            ),
        ],
    );
}
