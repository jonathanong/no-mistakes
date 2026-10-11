use super::*;

#[test]
fn append_mutations_do_not_reuse_stale_builder_alias_variants() {
    let calls = calls("variants-append-aliases.ts");
    assert_eq!(calls.len(), 18);
    for index in [0, 2, 3, 4, 5, 8, 10, 11, 16] {
        assert!(
            calls[index].is_unanalyzable(),
            "call {index}: {:#?}",
            calls[index]
        );
    }
    for (index, statement, alternate) in [
        (1, "SELECT id FROM users OFFSET 7", "SELECT 2"),
        (7, "SELECT id FROM accounts OFFSET 10", "SELECT 8"),
        (17, "SELECT id FROM users OFFSET 14", "SELECT 18"),
    ] {
        assert_eq!(texts(&calls[index]), [statement, alternate], "call {index}");
    }
    assert_eq!(texts(&calls[6]), ["SELECT id FROM users", "SELECT 7"]);
    assert_eq!(texts(&calls[12]), ["SELECT id FROM users", "SELECT 14"]);
    assert_eq!(
        texts(&calls[13]),
        ["SELECT id FROM users OFFSET 13", "SELECT 15"]
    );
    assert_eq!(
        texts(&calls[14]),
        ["SELECT id FROM users LIMIT 1", "SELECT 16"]
    );
    // The legacy chain resolver does not resolve an identifier receiver.
    // Finite recovery must leave that existing nonbranching shape unchanged.
    assert_eq!(calls[15].kind, EmbeddedSqlKind::Dynamic);
    assert_eq!(calls[15].sql_text, None);
    assert!(calls[15].variants.is_empty());
}

#[test]
fn helper_parameter_effects_keep_their_existing_lexical_shadow_boundary() {
    let calls = calls("variants-append-alias-shadows.ts");
    assert_eq!(calls.len(), 1);
    assert_eq!(texts(&calls[0]), ["SELECT 1", "SELECT 2"]);
}

#[test]
fn effectful_switch_labels_and_discriminants_keep_executed_sql_opaque() {
    let calls = calls("variants-switch-effects.ts");
    assert_eq!(calls.len(), 8);
    for (index, call) in calls.iter().enumerate().take(5) {
        assert!(call.is_unanalyzable(), "call {index}: {call:#?}");
        assert!(call.sql_text.is_none(), "call {index}: {call:#?}");
    }
    assert_eq!(texts(&calls[5]), ["SELECT 40", "SELECT 41"]);
    assert!(calls[6].is_unanalyzable());
    assert!(calls[6].sql_text.is_none());
    assert_eq!(calls[7].kind, EmbeddedSqlKind::ImmutableLocal);
    assert_eq!(calls[7].sql_text.as_deref(), Some("SELECT 99"));
}
