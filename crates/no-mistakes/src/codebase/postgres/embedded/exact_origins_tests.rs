use super::*;

#[test]
fn exact_inline_origins_are_opt_in_and_distinguish_same_line_unicode_calls() {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/postgres-facts/grouping/fixture/exact-origins.ts");
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/postgres-facts/grouping/fixture/exact-origins.ts"
    ));
    let options = EmbeddedSqlOptions::configured("db", &["query".into()]).with_trusted_sql_tags(&[
        TrustedSqlTag {
            module: "db".into(),
            name: "sql".into(),
        },
    ]);
    crate::ast::with_program(&path, source, |program, _| {
        let legacy =
            extract_embedded_sql_from_program_with_demand(&path, program, source, &options, false);
        let off = extract_embedded_sql_from_program_with_projections(
            &path,
            program,
            source,
            &options,
            (false, false),
        );
        let mut on = extract_embedded_sql_from_program_with_projections(
            &path,
            program,
            source,
            &options,
            (false, true),
        );
        assert_eq!(off.calls, legacy.calls);
        assert_eq!(off.calls.len(), 4);
        assert!(off
            .calls
            .iter()
            .all(|call| call.sql_source_offsets.is_empty()));
        let expected = source
            .match_indices("SELECT DISTINCT")
            .map(|(offset, _)| offset as u32)
            .collect::<Vec<_>>();
        assert_eq!(expected.len(), 4);
        for (index, call) in on.calls.iter_mut().enumerate() {
            let sql = call.sql_text.as_deref().unwrap();
            assert_eq!(call.sql_source_offsets.len(), sql.len());
            assert_eq!(call.sql_source_offsets[0], expected[index]);
            assert_eq!(
                call.source_offset_at_sql_position(sql, 1, 1),
                Some(expected[index] as usize)
            );
            call.sql_source_offsets.clear();
        }
        assert_eq!(on.calls, legacy.calls);
        assert_eq!(off.calls[0].line, off.calls[1].line);
        assert_ne!(expected[0], expected[1]);
        let prefix = source[..expected[0] as usize].rsplit('\n').next().unwrap();
        assert_ne!(prefix.len(), prefix.chars().count());
    })
    .unwrap();
}

#[test]
fn exact_origins_match_expanded_sql_and_preserve_dynamic_templates() {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/postgres-facts/grouping/fixture/exact-origin-splices.ts");
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/postgres-facts/grouping/fixture/exact-origin-splices.ts"
    ));
    let options = EmbeddedSqlOptions::configured("db", &["query".into()]).with_trusted_sql_tags(&[
        TrustedSqlTag {
            module: "db".into(),
            name: "sql".into(),
        },
    ]);
    crate::ast::with_program(&path, source, |program, _| {
        let off = extract_embedded_sql_from_program_with_projections(
            &path,
            program,
            source,
            &options,
            (false, false),
        );
        let on = extract_embedded_sql_from_program_with_projections(
            &path,
            program,
            source,
            &options,
            (false, true),
        );
        assert_eq!(off.calls.len(), 4);
        assert_eq!(on.calls.len(), off.calls.len());
        assert_eq!(on.executor_bindings, off.executor_bindings);
        assert_eq!(on.matched_factory_names, off.matched_factory_names);
        assert_eq!(on.matched_type_names, off.matched_type_names);
        assert_eq!(on.path, off.path);
        assert_eq!(
            on.calls[0].sql_text.as_deref(),
            Some("SELECT DISTINCT id FROM posts")
        );
        assert!(!on.calls[0].sql_source_offsets.is_empty());
        let select_at = source.find("SELECT DISTINCT id FROM ${table}").unwrap();
        assert_eq!(on.calls[0].sql_source_offsets[0], select_at as u32);
        let dynamic = on.calls.last().unwrap();
        let legacy_dynamic = off.calls.last().unwrap();
        assert_eq!(dynamic.sql_text, legacy_dynamic.sql_text);
        assert_eq!(dynamic.kind, legacy_dynamic.kind);
        assert_eq!(
            dynamic.sql_source_positions,
            legacy_dynamic.sql_source_positions
        );
        if !legacy_dynamic.sql_source_offsets.is_empty() {
            assert_eq!(
                dynamic.sql_source_offsets,
                legacy_dynamic.sql_source_offsets
            );
        } else {
            let dynamic_at = source
                .find("SELECT DISTINCT id FROM posts ${unresolved()}")
                .unwrap();
            assert_eq!(
                dynamic.sql_source_offsets.first().copied(),
                Some(dynamic_at as u32)
            );
        }
        let table_at = source.find("\"posts\"").unwrap() + 1;
        let call = &on.calls[0];
        let sql = call.sql_text.as_deref().unwrap();
        assert_eq!(
            call.source_offset_at_sql_offset(sql.find("posts").unwrap()),
            Some(table_at)
        );
        for (actual, original) in on.calls.iter().zip(&off.calls) {
            if !original.sql_source_offsets.is_empty() {
                assert_eq!(actual.sql_source_offsets, original.sql_source_offsets);
            }
            if !actual.sql_source_offsets.is_empty() {
                assert_eq!(
                    actual.sql_source_offsets.len(),
                    actual.sql_text.as_ref().unwrap().len()
                );
            }
            let mut restored = actual.clone();
            restored.sql_source_offsets = original.sql_source_offsets.clone();
            assert_eq!(restored, *original);
        }
    })
    .unwrap();
}
