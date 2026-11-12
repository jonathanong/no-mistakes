use super::*;

#[test]
fn failed_selected_operands_keep_helper_recovery_uncertain_without_running_other_arms() {
    assert_recovery(
        "3304-helper-failure-branches.mts",
        &[
            ("trueControl", Some("SELECT 1")),
            ("falseControl", Some("SELECT 2")),
            ("trueFailure", None),
            ("falseFailure", None),
            ("conditionFailure", None),
            ("memberFailure", None),
            ("computedObjectFailure", None),
            ("computedKeyFailure", None),
            ("appendObjectFailure", None),
            ("appendSuffixFailure", None),
        ],
    );
}

#[test]
fn one_failed_conditional_arm_keeps_other_sql_evidence_with_residual_uncertainty() {
    assert_recovery_mode(
        "3304-helper-partial-branches.mts",
        &[
            ("uncertainYes", Some("SELECT 1")),
            ("uncertainNo", Some("SELECT 1")),
        ],
        true,
    );
}
