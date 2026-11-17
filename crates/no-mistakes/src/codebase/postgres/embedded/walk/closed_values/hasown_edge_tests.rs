use super::hasown_tests::{calls, sqls};

#[test]
fn hasown_edges_keep_malformed_records_unanalyzable() {
    let calls = calls("hasown-edges.ts");
    assert_eq!(
        sqls(&calls, "/* cooked */"),
        [
            "/* cooked */ DELETE FROM users WHERE id = $1",
            "/* cooked */ DELETE FROM posts WHERE id = $1",
        ]
    );
    for marker in [
        "/* mixed-kinds */",
        "/* getter */",
        "/* empty-key */",
        "/* empty-nested */",
        "/* duplicate */",
        "/* uneven */",
        "/* blank */",
        "/* frozen-null */",
        "/* rest */",
        "/* default */",
        "/* pattern */",
        "/* typeof */",
        "/* plain */",
        "/* object-ns */",
        "/* string-arg */",
        "/* mismatch */",
        "/* static */",
        "/* ns-member */",
        "/* literal-key */",
        "/* optional-return */",
    ] {
        let call = calls
            .iter()
            .find(|call| {
                call.sql_text
                    .as_deref()
                    .is_some_and(|sql| sql.contains(marker))
                    || call
                        .variants
                        .iter()
                        .any(|variant| variant.sql_text.contains(marker))
            })
            .unwrap_or_else(|| panic!("missing {marker}: {calls:#?}"));
        assert!(call.is_unanalyzable(), "{marker}: {call:#?}");
    }
}

#[test]
fn hasown_guard_gaps_abstain_on_rejected_shapes_and_keep_early_returns() {
    let calls = calls("hasown-guard-gaps.ts");
    for marker in [
        "/* not-call */",
        "/* extra-stmt */",
        "/* and-guard */",
        "/* equal-typeof */",
        "/* bare-compare */",
        "/* void-typeof */",
        "/* member-typeof */",
        "/* other-typeof */",
        "/* ident-typeof */",
        "/* bare-block */",
        "/* expr-guard */",
        "/* empty-record */",
        "/* wide-record */",
        "/* many-records */",
    ] {
        let call = calls
            .iter()
            .find(|call| {
                call.sql_text
                    .as_deref()
                    .is_some_and(|sql| sql.contains(marker))
                    || call
                        .variants
                        .iter()
                        .any(|variant| variant.sql_text.contains(marker))
            })
            .unwrap_or_else(|| panic!("missing {marker}: {calls:#?}"));
        assert!(call.is_unanalyzable(), "{marker}: {call:#?}");
    }
    for marker in [
        "/* null-return */",
        "/* void-return */",
        "/* side-member */",
    ] {
        assert_eq!(
            sqls(&calls, marker),
            [
                format!("{marker} DELETE FROM users WHERE id = $1"),
                format!("{marker} DELETE FROM posts WHERE id = $1"),
            ]
        );
    }
}
