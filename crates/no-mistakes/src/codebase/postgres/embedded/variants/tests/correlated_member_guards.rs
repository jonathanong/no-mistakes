use super::*;

#[test]
fn closed_member_and_const_alias_keep_six_feasible_sql_variants() {
    let calls = calls("variants-correlated-member-guards.ts");
    assert_eq!(calls.len(), 7);

    for (call, description) in [(&calls[0], "direct member"), (&calls[1], "const alias")] {
        let mut variants = texts(call);
        variants.sort_unstable();
        let mut expected = [
                "/* selected/read */ WITH selected AS (SELECT id FROM own_items UNION ALL SELECT id FROM team_items) SELECT id FROM selected ORDER BY id",
                "/* selected/read */ WITH selected AS (SELECT id FROM own_items UNION ALL SELECT id FROM team_items) SELECT id FROM selected",
                "/* selected/read */ WITH selected AS (SELECT id FROM own_items) SELECT id FROM selected ORDER BY id",
                "/* selected/read */ WITH selected AS (SELECT id FROM own_items) SELECT id FROM selected",
                "/* selected/read */ WITH selected AS (SELECT id FROM team_items) SELECT id FROM selected ORDER BY id",
                "/* selected/read */ WITH selected AS (SELECT id FROM team_items) SELECT id FROM selected",
        ];
        expected.sort_unstable();
        assert_eq!(variants, expected, "{description}");
        assert!(!call.variants_truncated(), "{description}");
    }
}
