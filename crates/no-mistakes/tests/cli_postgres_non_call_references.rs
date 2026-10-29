use super::check_json_snapshot::{assert_saved_fixture, rules_path};

// A reference to a configured executor is not a call: only invocations report.
#[test]
fn executor_references_are_not_calls_check_json_snapshot() {
    let state = |name: &str| {
        rules_path(
            "postgres-no-test-state-mutation",
            "non-call-references",
            name,
        )
    };
    let annotation =
        |kind: &str, name: &str| rules_path("postgres-require-query-annotation", kind, name);
    assert_saved_fixture(
        "postgres_executor_non_call_references",
        &[
            ("state_low", state(""), state("low.yml")),
            ("state_max", state(""), state("max.yml")),
            (
                "annotation",
                annotation("fixture", "non-call-references"),
                annotation("configs", "non-call-references.yml"),
            ),
        ],
    );
}
