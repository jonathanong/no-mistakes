use super::*;

#[test]
fn later_arguments_update_the_captured_builder_but_never_recapture_primitive_values() {
    let file = file_facts("variants-executor-argument-effects.ts");
    let source = std::fs::read_to_string(&file.path).unwrap();
    let expected = source
        .lines()
        .enumerate()
        .filter_map(|(index, line)| {
            line.split_once("// ")
                .map(|(_, expectation)| (index as u32 + 1, expectation))
        })
        .collect::<Vec<_>>();
    assert_eq!(file.calls.len(), expected.len(), "{:#?}", file.calls);
    for (call, (line, expectation)) in file.calls.iter().zip(expected) {
        assert_eq!(call.line, line, "{call:#?}");
        if let Some(sql) = expectation.strip_prefix("variants: ") {
            let mut expected = sql.split(" | ").collect::<Vec<_>>();
            let mut actual = texts(call);
            expected.sort_unstable();
            actual.sort_unstable();
            assert_eq!(actual, expected, "{call:#?}");
        } else if expectation == "opaque" {
            assert!(call.is_unanalyzable(), "{call:#?}");
            assert_eq!(call.sql_text, None, "{call:#?}");
        } else {
            let (kind, sql) = expectation.split_once(": ").unwrap();
            let expected_kind = match kind {
                "unchanged" => EmbeddedSqlKind::ImmutableLocal,
                "composed" => EmbeddedSqlKind::Composed,
                "inline" => EmbeddedSqlKind::Inline,
                _ => panic!("unexpected expectation: {expectation}"),
            };
            assert_eq!(call.kind, expected_kind, "{call:#?}");
            assert_eq!(call.sql_text.as_deref(), Some(sql));
            assert!(call.variants.is_empty(), "{call:#?}");
        }
    }
    assert_eq!(file.calls, calls("variants-executor-argument-effects.ts"));
}

#[test]
fn provisional_executor_arguments_apply_the_same_captured_builder_effects() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres-facts/embedded/variants-executor-argument-relative.ts");
    let source = std::fs::read_to_string(&path).unwrap();
    let options = EmbeddedSqlOptions::configured("@example/db", &[])
        .with_scoped_executors(&["openTransaction".into()], &[]);
    let facts = extract_embedded_sql_from_source(&path, &source, &options);
    assert!(facts.calls.is_empty(), "{facts:#?}");
    assert_eq!(facts.pending_relative.calls.len(), 1, "{facts:#?}");
    let mut actual = texts(&facts.pending_relative.calls[0].call);
    actual.sort_unstable();
    assert_eq!(
        actual,
        [
            "SELECT id FROM relative_users OFFSET 1",
            "SELECT id FROM relative_users WHERE id = 1 OFFSET 1",
        ]
    );
}
