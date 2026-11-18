use super::*;

#[test]
fn named_member_guard_keeps_optional_tail_for_mine_and_team() {
    let calls = calls("variants-named-member-guards.ts");
    assert_eq!(calls.len(), 2);

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
fn named_members_with_the_same_type_keep_independent_choices() {
    let calls = calls("variants-named-member-guards.ts");
    assert_eq!(calls.len(), 2);

    let mut variants = texts(&calls[1]);
    variants.sort_unstable();
    let expected = [
        "WITH selected AS (SELECT id FROM own_items UNION ALL SELECT id FROM team_items) SELECT id FROM selected",
        "WITH selected AS (SELECT id FROM own_items) SELECT id FROM selected",
        "WITH selected AS (SELECT id FROM team_items) SELECT id FROM selected",
    ]
    .into_iter()
    .flat_map(|query| ["", " LIMIT 1", " LIMIT 2"].map(|tail| format!("{query}{tail}")))
    .collect::<Vec<_>>();
    assert_eq!(variants, expected);
    assert!(!calls[1].variants_truncated());
}
