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
fn passing_an_imported_executor_does_not_downgrade_its_calls() {
    let options =
        EmbeddedSqlOptions::configured("@example/db", &["pool".into(), "replacement".into()]);
    assert_sites(
        "review-imported-pool-pass.ts",
        &options,
        &[
            site(
                7,
                "pool.query('SELECT id FROM kept')",
                Some("SELECT id FROM kept"),
            ),
            site(15, "replacement.query('SELECT id FROM lost')", None),
        ],
    );
}

#[test]
fn namespace_alias_after_a_same_function_escape_stays_recorded() {
    assert_sites(
        "review-namespace-alias-ordered.ts",
        &scoped_options(),
        &[
            site(
                9,
                "db.query('SELECT id FROM alias_ordered')",
                Some("SELECT id FROM alias_ordered"),
            ),
            site(
                11,
                "query('SELECT id FROM destructure_ordered')",
                Some("SELECT id FROM destructure_ordered"),
            ),
        ],
    );
}

#[test]
fn namespace_alias_in_another_function_is_downgraded_not_dropped() {
    assert_sites(
        "review-namespace-alias-escaped.ts",
        &scoped_options(),
        &[
            site(13, "db.query('SELECT id FROM alias_escaped')", None),
            site(15, "query('SELECT id FROM destructure_escaped')", None),
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

#[test]
fn namespace_export_writes_share_named_import_origin() {
    let options = EmbeddedSqlOptions::configured(
        "@example/db",
        &["connection".into(), "other".into(), "query".into()],
    );
    assert_sites(
        "review-cross-form-export-write.ts",
        &options,
        &[
            site(
                7,
                "left.query('SELECT id FROM before_export_write')",
                Some("SELECT id FROM before_export_write"),
            ),
            site(9, "left.query('SELECT id FROM after_export_write')", None),
            site(
                10,
                "other.query('SELECT id FROM other_export_unaffected')",
                Some("SELECT id FROM other_export_unaffected"),
            ),
            site(
                12,
                "left.query('SELECT id FROM after_computed_export_write')",
                None,
            ),
        ],
    );
}

#[test]
fn namespace_export_and_whole_namespace_transfer_have_distinct_reach() {
    let options = EmbeddedSqlOptions::configured(
        "@example/db",
        &["connection".into(), "other".into(), "query".into()],
    );
    assert_sites(
        "review-cross-form-export-transfer.ts",
        &options,
        &[
            site(
                8,
                "left.query('SELECT id FROM before_export_transfer')",
                Some("SELECT id FROM before_export_transfer"),
            ),
            site(
                10,
                "left.query('SELECT id FROM after_export_transfer')",
                Some("SELECT id FROM after_export_transfer"),
            ),
            site(
                11,
                "other.query('SELECT id FROM other_export_before_namespace_transfer')",
                Some("SELECT id FROM other_export_before_namespace_transfer"),
            ),
            site(
                13,
                "other.query('SELECT id FROM other_export_after_namespace_transfer')",
                None,
            ),
        ],
    );
}
