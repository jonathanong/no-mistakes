#[test]
fn cross_module_append_keeps_for_update_parseable() {
    let sql = sql_by_file(&project_case(
        "postgres-lock-ordering",
        "cross-module-claim",
    ));
    let claim = &sql["claim.ts"];
    assert!(claim.iter().any(|sql| {
        sql.contains("failed_id IS NULL")
            && sql.contains("FOR UPDATE OF work")
            && sql.contains("work.id = ANY($1::uuid[])")
            && sql.contains("attempt_count < $2")
            && sql.contains("GREATEST($3::timestamptz, clock_timestamp())")
            && sql.contains("lease_expires_at <= $4")
            && sql.contains("now() + $5 * interval")
            && !sql.contains("sql_placeholder_")
            && !sql.contains("30000")
            && !sql.contains("AND  ORDER")
    }));
    assert!(claim
        .iter()
        .any(|sql| sql == "SELECT work.id FROM work WHERE work.id = ANY($1::uuid[]) ORDER BY work.record_id FOR UPDATE OF work"));
}

#[test]
fn statement_append_keeps_both_arms() {
    let sql = sql_by_file(&project_case("postgres-lock-ordering", "statement-append"));
    let lock = &sql["lock.ts"];
    assert!(lock.iter().any(|sql| {
        sql.contains("placement.a_id = $1")
            && sql.contains("surface.image_id = $2 ORDER BY placement.id LIMIT 1 FOR UPDATE OF placement")
    }));
    assert!(lock.iter().any(|sql| {
        sql.contains("placement.b_id = $1")
            && sql.contains("surface.image_id = $2 ORDER BY placement.id LIMIT 1 FOR UPDATE OF placement")
    }));
    assert!(lock
        .iter()
        .any(|sql| sql == "SELECT placement.id FROM placements placement WHERE placement.id = ANY($1::uuid[]) ORDER BY placement.surface_id FOR UPDATE OF placement"));
}

#[test]
fn plain_string_append_splices_raw_sql() {
    let sql = sql_by_file(&project_case(
        "postgres-required-predicates",
        "raw-column-append",
    ));
    let list = &sql["list.ts"];
    assert!(list.len() >= 2, "{list:?}");
    for text in list {
        assert!(text.contains("s.item_id = c.id"), "{text}");
        assert!(!text.contains("s.item_id = sql_placeholder"), "{text}");
        assert!(text.contains("AND s.visible"), "{text}");
    }
}

#[test]
fn imported_numeric_limit_stays_a_bind() {
    let files = project_case("postgres-conflict-ordering", "imported-limit");
    for (_, file) in &files {
        for call in &file.embedded_sql[0].1.calls {
            assert_eq!(call.kind, EmbeddedSqlKind::Inline, "{call:?}");
        }
    }
    let insert = &sql_by_file(&files)["insert.ts"];
    assert!(insert.len() >= 2, "{insert:?}");
    for text in insert {
        assert!(text.contains("LIMIT $4"), "{text}");
        assert!(
            text.contains("ON CONFLICT (conversation_id, user_id)"),
            "{text}"
        );
        assert!(!text.contains("100"), "{text}");
        assert!(text.contains("unnest($1::uuid[], $2::text[])"), "{text}");
        assert!(text.contains("SELECT $5, r.user_id, r.role"), "{text}");
        assert!(!text.contains("sql_placeholder_"), "{text}");
    }
    let root_cause = &sql_by_file(&files)["root-cause.ts"];
    assert_eq!(root_cause.len(), 2, "{root_cause:?}");
    for text in root_cause {
        assert!(text.contains("LIMIT $4"), "{text}");
        assert!(text.contains("generate_series(1, $5)"), "{text}");
        assert!(text.contains("SELECT $6, i.user_id, i.role FROM input i"), "{text}");
        assert!(text.contains("ON CONFLICT DO NOTHING RETURNING user_id"), "{text}");
        assert!(!text.contains("100") && !text.contains("sql_placeholder_"), "{text}");
    }
}
