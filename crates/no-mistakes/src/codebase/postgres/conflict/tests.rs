use super::*;

#[test]
fn preserves_expression_conflict_targets_while_parsing_the_source_order() {
    let inserts = analyze_conflict_inserts(
        "INSERT INTO categories (item_id, category) SELECT item_id, category FROM input ORDER BY item_id, lower(category) ON CONFLICT (item_id, (LOWER(category))) DO NOTHING",
    )
    .unwrap();
    assert_eq!(inserts.len(), 1);
    assert_eq!(
        inserts[0].target,
        SqlConflictTarget::Columns {
            expressions: vec!["item_id".to_string(), "(LOWER(category))".to_string()],
            predicate: None,
        }
    );
    assert!(inserts[0].source.multi_row);
    assert_eq!(inserts[0].source.order.as_ref().unwrap().len(), 2);
}

#[test]
fn marks_targetless_conflicts() {
    let inserts = analyze_conflict_inserts(
        "INSERT INTO items (id) SELECT id FROM input ORDER BY id ON CONFLICT DO NOTHING",
    )
    .unwrap();
    assert_eq!(inserts[0].target, SqlConflictTarget::Targetless);
}

#[test]
fn preserves_partial_index_predicates() {
    let inserts = analyze_conflict_inserts(
        "INSERT INTO items (id) SELECT id FROM input ORDER BY id ON CONFLICT (id) WHERE is_live DO NOTHING",
    )
    .unwrap();
    assert_eq!(
        inserts[0].target,
        SqlConflictTarget::Columns {
            expressions: vec!["id".to_string()],
            predicate: Some("is_live".to_string()),
        }
    );

    let after_action = analyze_conflict_inserts(
        "INSERT INTO items (id) SELECT id FROM input ON CONFLICT (id) DO UPDATE SET id = EXCLUDED.id WHERE id > 0",
    )
    .unwrap();
    assert_eq!(
        after_action[0].target,
        SqlConflictTarget::Columns {
            expressions: vec!["id".to_string()],
            predicate: None,
        }
    );
}

#[test]
fn ignores_keywords_in_strings_comments_and_identifiers() {
    let sql = r#"-- ON CONFLICT in a line comment
        SELECT 'ON CONFLICT'; /* ON CONFLICT in a block comment */
        SELECT on_conflict FROM records"#;
    assert!(analyze_conflict_inserts(sql).unwrap().is_empty());
    assert!(raw::raw_conflicts(sql).unwrap().is_empty());
    assert!(raw::raw_conflicts("/* ON CONFLICT").unwrap().is_empty());
    assert!(raw::raw_conflicts("SELECT 'ON CONFLICT")
        .unwrap()
        .is_empty());
    assert!(raw::raw_conflicts("SELECT 1 ON foo").unwrap().is_empty());
    assert!(
        raw::raw_conflicts("SELECT 1 ON something CONFLICT DO NOTHING")
            .unwrap()
            .is_empty()
    );
}

#[test]
fn recognizes_constraint_targets_and_sanitizes_column_targets() {
    let constraint = raw::raw_conflicts(
        "INSERT INTO items VALUES (1) ON CONFLICT ON CONSTRAINT \"items_pkey\" DO NOTHING",
    )
    .unwrap();
    assert_eq!(constraint.len(), 1);
    assert_eq!(
        constraint[0].target,
        SqlConflictTarget::Constraint("\"items_pkey\"".into())
    );
    assert_eq!(raw::sanitize("SELECT 1", &constraint), "SELECT 1");

    let unquoted = raw::raw_conflicts(
        "INSERT INTO items VALUES (1) ON CONFLICT ON CONSTRAINT public.items_pkey DO NOTHING",
    )
    .unwrap();
    assert_eq!(
        unquoted[0].target,
        SqlConflictTarget::Constraint("public.items_pkey".into())
    );

    let at_start = raw::raw_conflicts("ON CONFLICT DO NOTHING").unwrap();
    assert_eq!(at_start[0].target, SqlConflictTarget::Targetless);

    let columns = raw::raw_conflicts(
        "INSERT INTO items VALUES (1) ON CONFLICT (id, (COALESCE(name, 'x)')),) WHERE active DO NOTHING",
    )
    .unwrap();
    assert_eq!(
        raw::sanitize(
            "INSERT INTO items VALUES (1) ON CONFLICT (id, (COALESCE(name, 'x)')),) WHERE active DO NOTHING",
            &columns,
        ),
        "INSERT INTO items VALUES (1) ON CONFLICT (nm_conflict_key)  DO NOTHING"
    );
}

#[test]
fn reports_malformed_constraint_and_column_targets() {
    for (sql, expected) in [
        (
            "INSERT INTO items VALUES (1) ON CONFLICT ON CONSTRAINT",
            "no constraint name",
        ),
        (
            "INSERT INTO items VALUES (1) ON CONFLICT ON CONSTRAINT \"items_pkey DO NOTHING",
            "unclosed constraint identifier",
        ),
        (
            "INSERT INTO items VALUES (1) ON CONFLICT (id",
            "unclosed ON CONFLICT target",
        ),
        (
            "INSERT INTO items VALUES (1) ON CONFLICT (id)",
            "has no DO action",
        ),
    ] {
        let error = analyze_conflict_inserts(sql).unwrap_err();
        assert!(error.to_string().contains(expected), "{sql}: {error:#}");
    }
}

#[test]
fn captures_values_cardinality_and_default_values() {
    let inserts = analyze_conflict_inserts(
        "INSERT INTO items (id) VALUES (1) ON CONFLICT DO NOTHING;
         INSERT INTO items (id) VALUES (1), (2) ON CONFLICT DO NOTHING;
         INSERT INTO items DEFAULT VALUES ON CONFLICT DO NOTHING;
         INSERT INTO items (id) (SELECT id FROM input) ON CONFLICT DO NOTHING;
         INSERT INTO items (id) SELECT id FROM first_input UNION ALL SELECT id FROM second_input ON CONFLICT DO NOTHING",
    )
    .unwrap();
    assert_eq!(inserts.len(), 5);
    assert!(!inserts[0].source.multi_row);
    assert!(inserts[1].source.multi_row);
    assert_eq!(inserts[0].source.projections, None);
    assert_eq!(
        inserts[2].source,
        SqlInsertSourceShape {
            multi_row: false,
            order: None,
            projections: None,
            order_aliases: Default::default(),
        }
    );
    assert!(inserts[3].source.multi_row);
    assert!(inserts[4].source.multi_row);
}

#[test]
fn captures_projection_aliases_and_ordering_defaults() {
    let inserts = analyze_conflict_inserts(
        "INSERT INTO items (ID, note)
         SELECT id AS source_id, note FROM input
         ORDER BY source_id DESC NULLS FIRST, note ASC NULLS LAST
         ON CONFLICT (id) DO NOTHING",
    )
    .unwrap();
    let source = &inserts[0].source;
    assert_eq!(
        source.projections.as_ref().unwrap().get("id"),
        Some(&"id".into())
    );
    assert_eq!(source.order_aliases.get("source_id"), Some(&"id".into()));
    assert_eq!(
        source.order,
        Some(vec![
            CanonicalOrderKey {
                expression: "source_id".into(),
                ascending: false,
                nulls_first: true,
            },
            CanonicalOrderKey {
                expression: "note".into(),
                ascending: true,
                nulls_first: false,
            },
        ])
    );
}

#[test]
fn declines_non_expression_order_and_projection_shapes() {
    let all_order = analyze_conflict_inserts(
        "INSERT INTO items SELECT * FROM input ORDER BY ALL ON CONFLICT DO NOTHING",
    )
    .unwrap();
    assert_eq!(
        all_order[0].source.order,
        Some(vec![CanonicalOrderKey {
            expression: "ALL".into(),
            ascending: true,
            nulls_first: false,
        }])
    );
    assert_eq!(all_order[0].source.projections, None);
    assert!(all_order[0].source.order_aliases.is_empty());

    let mismatch = analyze_conflict_inserts(
        "INSERT INTO items (id, note) SELECT id FROM input ON CONFLICT DO NOTHING",
    )
    .unwrap();
    assert_eq!(mismatch[0].source.projections, None);

    let wildcard = analyze_conflict_inserts(
        "INSERT INTO items (id) SELECT * FROM input ON CONFLICT DO NOTHING",
    )
    .unwrap();
    assert_eq!(wildcard[0].source.projections, None);
}

#[test]
fn order_by_all_is_ignored_directly() {
    let order = sqlparser::ast::OrderBy {
        kind: sqlparser::ast::OrderByKind::All(Default::default()),
        interpolate: None,
    };
    assert_eq!(order_keys(&order), None);
}

#[test]
fn skips_non_insert_and_non_conflict_statements() {
    let inserts = analyze_conflict_inserts(
        "SELECT 1;
         INSERT INTO items VALUES (1);
         UPDATE items SET id = 2;
         INSERT INTO items VALUES (1) ON CONFLICT DO NOTHING",
    )
    .unwrap();
    assert_eq!(inserts.len(), 1);
    assert_eq!(inserts[0].target, SqlConflictTarget::Targetless);
}

#[test]
fn skips_tagged_and_untagged_dollar_quoted_bodies() {
    let sql = r#"
        SELECT $$ INSERT INTO decoy VALUES (1) ON CONFLICT (decoy_id) DO NOTHING $$;
        SELECT $body$ ON CONFLICT (also_decoy) DO NOTHING $body$;
        INSERT INTO items (id) VALUES (1) ON CONFLICT (id) DO NOTHING
    "#;
    let raw = raw::raw_conflicts(sql).unwrap();
    assert_eq!(raw.len(), 1);
    assert_eq!(
        raw[0].target,
        SqlConflictTarget::Columns {
            expressions: vec!["id".into()],
            predicate: None,
        }
    );
    let inserts = analyze_conflict_inserts(sql).unwrap();
    assert_eq!(inserts.len(), 1);
    assert_eq!(inserts[0].table, "items");
}

#[test]
fn skips_unterminated_dollar_quoted_body() {
    let sql = "SELECT $function$ INSERT INTO decoy ON CONFLICT (id) DO NOTHING";
    assert!(raw::raw_conflicts(sql).unwrap().is_empty());
    assert!(analyze_conflict_inserts(sql).unwrap().is_empty());
    assert_eq!(
        raw::raw_conflicts("SELECT $tag INSERT ON CONFLICT")
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn traverses_data_modifying_ctes_before_the_outer_statement() {
    let sql = "WITH inserted AS (
        INSERT INTO items (id) VALUES (1) ON CONFLICT (id) DO NOTHING RETURNING id
    )
    SELECT id FROM inserted";
    let inserts = analyze_conflict_inserts(sql).unwrap();
    assert_eq!(inserts.len(), 1);
    assert_eq!(inserts[0].table, "items");
    assert_eq!(
        inserts[0].target,
        SqlConflictTarget::Columns {
            expressions: vec!["id".into()],
            predicate: None,
        }
    );
}

#[test]
fn preserves_raw_target_alignment_across_ctes_and_outer_insert() {
    let sql = "WITH inserted AS (
        INSERT INTO items (id) VALUES (1) ON CONFLICT (id) DO NOTHING RETURNING id
    )
    INSERT INTO summaries (id) SELECT id FROM inserted ON CONFLICT (id) DO NOTHING";
    let inserts = analyze_conflict_inserts(sql).unwrap();
    assert_eq!(inserts.len(), 2);
    assert_eq!(inserts[0].table, "items");
    assert_eq!(inserts[1].table, "summaries");
}

#[test]
fn analyzes_default_values_and_union_sources() {
    let defaults =
        analyze_conflict_inserts("INSERT INTO items DEFAULT VALUES ON CONFLICT DO NOTHING")
            .unwrap();
    assert_eq!(defaults.len(), 1);
    assert!(!defaults[0].source.multi_row);

    let union = analyze_conflict_inserts(
        "INSERT INTO items (id) SELECT id FROM input UNION SELECT id FROM extra ON CONFLICT (id) DO NOTHING",
    );
    assert!(union.is_ok(), "{union:?}");
}

#[test]
fn remaining_set_expr_and_insert_shapes() {
    let nested = analyze_conflict_inserts(
        "INSERT INTO items (id) SELECT id FROM (
            INSERT INTO nested (id) VALUES (1) ON CONFLICT (id) DO NOTHING RETURNING id
         ) AS src ON CONFLICT (id) DO NOTHING",
    );
    assert!(
        nested
            .unwrap_err()
            .to_string()
            .contains("could not align ON CONFLICT"),
        "nested INSERT..ON CONFLICT should fail alignment"
    );

    let except = analyze_conflict_inserts(
        "INSERT INTO items (id) SELECT id FROM first EXCEPT SELECT id FROM second ON CONFLICT (id) DO NOTHING",
    );
    assert!(except.is_ok(), "{except:?}");

    let intersecting = analyze_conflict_inserts(
        "INSERT INTO items (id) SELECT id FROM first INTERSECT SELECT id FROM extra ON CONFLICT (id) DO NOTHING",
    );
    assert!(intersecting.is_ok(), "{intersecting:?}");

    let values_query = analyze_conflict_inserts(
        "INSERT INTO items (id) (VALUES (1), (2)) ON CONFLICT (id) DO NOTHING",
    )
    .unwrap();
    assert!(values_query[0].source.multi_row);

    let Statement::Query(query) = parse_postgres_sql("SELECT 1 UNION SELECT 2")
        .unwrap()
        .pop()
        .unwrap()
    else {
        panic!("union query");
    };
    assert!(query_is_potentially_multi_row(query.body.as_ref()));

    let Statement::Query(wrapped) = parse_postgres_sql("(SELECT 1)").unwrap().pop().unwrap() else {
        panic!("wrapped");
    };
    assert!(query_is_potentially_multi_row(wrapped.body.as_ref()));

    let insert_stmt = parse_postgres_sql("INSERT INTO items VALUES (1)")
        .unwrap()
        .pop()
        .unwrap();
    let mut raw = Vec::new().into_iter();
    let mut inserts = Vec::new();
    collect_statement(&insert_stmt, &mut raw, &mut inserts).unwrap();
    assert!(inserts.is_empty());

    let update = parse_postgres_sql("UPDATE items SET id = 1")
        .unwrap()
        .pop()
        .unwrap();
    collect_statement(&update, &mut Vec::new().into_iter(), &mut inserts).unwrap();
}
