use super::*;

#[test]
fn direct_helper_mutations_invalidate_captured_builder_aliases_after_argument_evaluation() {
    let calls = calls("variants-helper-effects.ts");
    assert_eq!(calls.len(), 33);
    assert_eq!(
        texts(&calls[0]),
        ["SELECT id FROM users OFFSET 7", "SELECT 2"]
    );
    for (index, prefix) in [
        (1, Some("SELECT id FROM users")),
        (2, None),
        (8, None),
        (11, Some("SELECT id FROM incomplete_users")),
    ] {
        assert_eq!(calls[index].kind, EmbeddedSqlKind::Dynamic, "call {index}");
        assert!(calls[index].variants.is_empty(), "call {index}");
        assert_eq!(calls[index].sql_text.as_deref(), prefix, "call {index}");
    }
    assert_eq!(calls[3].kind, EmbeddedSqlKind::ImmutableLocal);
    assert_eq!(
        calls[3].sql_text.as_deref(),
        Some("SELECT id FROM accounts")
    );
    assert_eq!(texts(&calls[4]), ["SELECT id FROM accounts", "SELECT 3"]);
    assert!(calls[5].is_unanalyzable());
    assert_eq!(calls[6].kind, EmbeddedSqlKind::ImmutableLocal);
    assert_eq!(calls[6].sql_text.as_deref(), Some("SELECT id FROM orders"));
    assert!(calls[7].is_unanalyzable());
    assert_eq!(texts(&calls[9]), ["SELECT id FROM new_users", "SELECT 5"]);
    assert!(calls[10].is_unanalyzable());
    assert_eq!(calls[12].kind, EmbeddedSqlKind::Composed);
    assert_eq!(
        calls[12].sql_text.as_deref(),
        Some("SELECT id FROM recent_users OFFSET 7 LIMIT 2")
    );
    assert!(calls[12].variants.is_empty());
    assert!(calls[12]
        .sql_source_positions
        .iter()
        .any(|position| position.source_line == 5));
    assert_eq!(texts(&calls[14]), ["SELECT id FROM pure_users", "SELECT 7"]);
    assert_eq!(texts(&calls[15]), ["SELECT id FROM pure_users", "SELECT 8"]);
    assert_eq!(
        texts(&calls[16]),
        ["SELECT id FROM redeclared_users", "SELECT 9"]
    );
    assert!(calls[18].is_unanalyzable());
    assert_eq!(
        texts(&calls[19]),
        ["SELECT id FROM append_users WHERE active", "SELECT 11"]
    );
    assert!(calls[21].is_unanalyzable());
    assert_eq!(
        texts(&calls[22]),
        ["SELECT id FROM passthrough_users", "SELECT 12"]
    );
    assert!(calls[24].is_unanalyzable());
    for (before, after, statement, alternative) in [
        (25, 26, "SELECT id FROM nested_users", "SELECT 13"),
        (27, 28, "SELECT id FROM other_users", "SELECT 14"),
        (29, 31, "SELECT id FROM first_users", "SELECT 15"),
        (30, 32, "SELECT id FROM second_users", "SELECT 16"),
    ] {
        assert_eq!(texts(&calls[before]), [statement, alternative]);
        assert!(
            calls[after].is_unanalyzable(),
            "call {after}: {:#?}",
            calls[after]
        );
    }
}

#[test]
fn helper_mutations_follow_historical_aliases_when_branch_reassignments_hide_identity() {
    let calls = calls("variants-helper-branch-effects.ts");
    assert_eq!(calls.len(), 7);
    assert_eq!(
        texts(&calls[0]),
        ["SELECT id FROM original_users", "SELECT 11"]
    );
    for index in [1, 2, 3, 5, 6] {
        assert!(
            calls[index].is_unanalyzable(),
            "call {index}: {:#?}",
            calls[index]
        );
    }
    assert_eq!(texts(&calls[4]), ["SELECT id FROM accounts", "SELECT 12"]);
}

#[test]
fn expression_effects_keep_outer_builder_snapshots_opaque_and_skip_unexecuted_callbacks() {
    let calls = calls("variants-expression-effects.ts");
    assert_eq!(calls.len(), 20);
    for (index, call) in calls.iter().enumerate().take(8) {
        assert!(call.is_unanalyzable(), "call {index}: {call:#?}");
    }
    assert_eq!(texts(&calls[8]), ["SELECT 9"]);
    assert_eq!(texts(&calls[9]), ["SELECT id FROM sole_users OFFSET 7"]);
    assert_eq!(texts(&calls[10]), ["SELECT 11", "SELECT 12"]);
    for (index, call) in calls.iter().enumerate().take(18).skip(11) {
        assert!(!call.is_unanalyzable(), "call {index}: {call:#?}");
    }
    assert!(calls[18].is_unanalyzable());
    assert_eq!(texts(&calls[19]), ["SELECT 28", "SELECT 29"]);
}

#[test]
fn helper_effects_track_fresh_constructor_identity_and_reject_missing_builder_arguments() {
    let calls = calls("variants-helper-constructor-effects.ts");
    assert_eq!(calls.len(), 8);
    for (index, statement, alternative) in [
        (0, "SELECT id FROM call_users", "SELECT 1"),
        (2, "SELECT id FROM raw_users", "SELECT 2"),
        (4, "SELECT id FROM join_users", "SELECT 3"),
    ] {
        assert_eq!(texts(&calls[index]), [statement, alternative]);
    }
    for index in [1, 3, 5, 6, 7] {
        assert!(
            calls[index].is_unanalyzable(),
            "call {index}: {:#?}",
            calls[index]
        );
    }
}
