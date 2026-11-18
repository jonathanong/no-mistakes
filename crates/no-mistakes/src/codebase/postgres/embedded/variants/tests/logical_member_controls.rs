use super::*;

#[test]
fn negated_unknown_tail_keeps_both_outcomes_for_compatible_modes() {
    let calls = calls("variants-logical-member-controls.ts");
    assert_eq!(calls.len(), 3);

    let mut variants = texts(&calls[0]);
    variants.sort_unstable();
    assert_eq!(
        variants,
        [
            "WITH selected AS (SELECT id FROM own_items UNION ALL SELECT id FROM team_items) SELECT id FROM selected",
            "WITH selected AS (SELECT id FROM own_items) SELECT id FROM selected",
            "WITH selected AS (SELECT id FROM own_items) SELECT id FROM selected ORDER BY id",
            "WITH selected AS (SELECT id FROM team_items) SELECT id FROM selected",
            "WITH selected AS (SELECT id FROM team_items) SELECT id FROM selected ORDER BY id",
        ]
    );
    assert!(!calls[0].variants_truncated());
}

#[test]
fn finite_member_truthiness_uses_the_value_selected_by_the_left_operand() {
    let calls = calls("variants-logical-member-controls.ts");
    assert_eq!(calls.len(), 3);

    let mut variants = texts(&calls[1]);
    variants.sort_unstable();
    assert_eq!(variants, ["SELECT 1 WHERE FALSE", "SELECT 1 WHERE TRUE"]);
    assert!(!calls[1].variants_truncated());
}

#[test]
fn logical_identifier_operand_shares_the_earlier_append_guard() {
    let calls = calls("variants-logical-member-controls.ts");
    assert_eq!(calls.len(), 3);

    let mut variants = texts(&calls[2]);
    variants.sort_unstable();
    assert_eq!(
        variants,
        [
            "SELECT 1",
            "SELECT 1 ORDER BY 1",
            "SELECT 1 ORDER BY 1 NULLS FIRST",
        ]
    );
    assert!(!calls[2].variants_truncated());
}
