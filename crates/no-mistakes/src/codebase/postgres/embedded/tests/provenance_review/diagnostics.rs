use super::*;

#[test]
fn same_commonjs_module_aliases_share_invalidation_but_subpath_does_not() {
    use EmbeddedSqlKind::{Dynamic, Inline};
    assert_sites(
        "review-namespace-identity.cjs",
        &scoped_options(),
        &[
            Site {
                line: 6,
                expression: "first.query('SELECT id FROM namespace_before')",
                sql: Some("SELECT id FROM namespace_before"),
                kind: Inline,
            },
            Site {
                line: 8,
                expression: "first.query('SELECT id FROM namespace_first_after')",
                sql: None,
                kind: Dynamic,
            },
            Site {
                line: 9,
                expression: "same.query('SELECT id FROM namespace_same_after')",
                sql: None,
                kind: Dynamic,
            },
            Site {
                line: 10,
                expression: "subpath.query('SELECT id FROM namespace_subpath')",
                sql: Some("SELECT id FROM namespace_subpath"),
                kind: Inline,
            },
        ],
    );
}

#[test]
fn deferred_closure_does_not_claim_pre_escape_certainty() {
    use EmbeddedSqlKind::{Dynamic, Inline};
    assert_sites(
        "review-deferred-closure.ts",
        &scoped_options(),
        &[
            Site {
                line: 4,
                expression: "tx.query('SELECT id FROM closure_before')",
                sql: Some("SELECT id FROM closure_before"),
                kind: Inline,
            },
            Site {
                line: 6,
                expression: "tx.query('SELECT id FROM closure_deferred')",
                sql: None,
                kind: Dynamic,
            },
        ],
    );
}

#[test]
fn unproven_sql_looking_member_is_counted_without_making_a_verdict() {
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let facts = crate::diagnostics::with_observer(Some(observer.clone()), || {
        extract_embedded_sql_from_source(
            &fixture("review-unproven-sql.ts"),
            &read_fixture("review-unproven-sql.ts"),
            &scoped_options(),
        )
    });
    assert!(facts.calls.is_empty());
    assert_eq!(
        observer
            .snapshot()
            .work
            .get("postgres.unproven_member_query_sql")
            .copied(),
        Some(1),
        "SQL-looking unproven query counts once; HTTP object query does not"
    );
}

#[test]
fn relative_member_counter_waits_for_resolution_even_after_escape() {
    use crate::codebase::postgres::embedded::relative::project_relative_scoped_facts;
    use std::path::Path;
    let options = scoped_options();
    for (resolved, expected) in [
        ("/virtual/db/transaction.ts", None),
        ("/virtual/other/transaction.ts", Some(1)),
    ] {
        let observer = crate::diagnostics::InvocationObserver::new(true);
        crate::diagnostics::with_observer(Some(observer.clone()), || {
            let mut facts = extract_embedded_sql_from_source(
                &fixture("relative-member-escape.ts"),
                &read_fixture("relative-member-escape.ts"),
                &options,
            );
            project_relative_scoped_facts(&mut facts, Some(Path::new("/virtual/db")), |_| {
                Some(PathBuf::from(resolved))
            });
        });
        assert_eq!(
            observer
                .snapshot()
                .work
                .get("postgres.unproven_member_query_sql")
                .copied(),
            expected,
            "resolution at {resolved} must decide whether the pending member is unproven"
        );
    }
}
