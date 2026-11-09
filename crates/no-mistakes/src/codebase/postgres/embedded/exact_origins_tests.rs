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
