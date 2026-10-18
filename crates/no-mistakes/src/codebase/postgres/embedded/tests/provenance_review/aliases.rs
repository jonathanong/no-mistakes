use super::*;

fn site(line: u32, expression: &'static str, sql: Option<&'static str>) -> Site<'static> {
    Site {
        line,
        expression,
        sql,
        kind: if sql.is_some() {
            EmbeddedSqlKind::Inline
        } else {
            EmbeddedSqlKind::Dynamic
        },
    }
}

#[test]
fn mutable_alias_branches_unknown_rebind_and_identity_assignment_are_sound() {
    assert_sites(
        "review-alias-branches.ts",
        &scoped_options(),
        &[
            site(8, "first.query('SELECT id FROM branch_first')", None),
            site(9, "other.query('SELECT id FROM branch_other')", None),
            site(
                10,
                "third.query('SELECT id FROM branch_third')",
                Some("SELECT id FROM branch_third"),
            ),
            site(17, "first.query('SELECT id FROM opaque_first')", None),
            site(
                22,
                "tx.query('SELECT id FROM self_assignment')",
                Some("SELECT id FROM self_assignment"),
            ),
            site(29, "first.query('SELECT id FROM loop_first')", None),
            site(30, "other.query('SELECT id FROM loop_other')", None),
        ],
    );
}

#[test]
fn two_named_import_aliases_share_one_export_origin() {
    let options =
        EmbeddedSqlOptions::configured("@example/db", &["connection".into(), "query".into()]);
    assert_sites(
        "review-imported-export-alias.ts",
        &options,
        &[
            site(
                4,
                "left.query('SELECT id FROM export_before')",
                Some("SELECT id FROM export_before"),
            ),
            site(6, "left.query('SELECT id FROM export_after')", None),
        ],
    );
}
