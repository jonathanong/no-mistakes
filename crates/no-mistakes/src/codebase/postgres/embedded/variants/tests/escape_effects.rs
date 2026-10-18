use super::*;

#[test]
fn unknown_helper_guards_and_escapes_never_publish_stale_builder_versions() {
    let file = file_facts("variants-unknown-helper-effects.ts");
    let source = std::fs::read_to_string(&file.path).unwrap();
    let expected = source
        .lines()
        .enumerate()
        .filter_map(|(index, line)| {
            line.split_once("// ")
                .map(|(_, expectation)| (index + 1, expectation))
        })
        .collect::<Vec<_>>();
    assert_eq!(file.calls.len(), expected.len(), "{:#?}", file.calls);
    for (call, (line, expectation)) in file.calls.iter().zip(expected) {
        assert_eq!(call.line, line as u32, "{call:#?}");
        match expectation {
            "opaque" | "opaque legacy-alias" => {
                assert!(call.is_unanalyzable(), "{call:#?}");
                assert_eq!(call.sql_text, None, "{call:#?}");
            }
            "opaque template" | "opaque known-prefix" => {
                assert!(call.is_unanalyzable(), "{call:#?}");
            }
            "finite" => {
                assert_eq!(call.variants.len(), 2, "{call:#?}");
            }
            "unchanged builder" | "unchanged string" => {
                assert_eq!(call.kind, EmbeddedSqlKind::ImmutableLocal, "{call:#?}");
                assert!(call.sql_text.is_some(), "{call:#?}");
                assert!(call.variants.is_empty(), "{call:#?}");
            }
            _ => panic!("unexpected fixture expectation: {expectation}"),
        }
    }
}

#[test]
fn confirmed_consumers_preserve_variants_while_pending_consumers_preserve_their_capture() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres-facts/embedded/variants-executor-builder-arguments.ts");
    let source = std::fs::read_to_string(&path).unwrap();
    let options = EmbeddedSqlOptions::configured("@example/db", &[])
        .with_scoped_executors(&["openTransaction".into()], &["TxExecutor".into()]);
    let facts = extract_embedded_sql_from_source(&path, &source, &options);
    assert_eq!(facts.calls.len(), 6, "{facts:#?}");
    for (index, call) in facts.calls.iter().enumerate() {
        if [2, 3].contains(&index) {
            // These reads follow an unconfirmed relative invocation.
            // Pending captures stay intact; later legacy and finite reuse
            // both become opaque. The unowned client.query is not SQL.
            assert!(call.is_unanalyzable(), "{call:#?}");
        } else if [1, 5].contains(&index) {
            assert_eq!(call.variants.len(), 2, "{call:#?}");
        } else {
            assert_eq!(call.kind, EmbeddedSqlKind::ImmutableLocal, "{call:#?}");
            assert!(call.variants.is_empty(), "{call:#?}");
        }
    }
    assert_eq!(facts.pending_relative.calls.len(), 1, "{facts:#?}");
    assert_eq!(
        facts.pending_relative.calls[0].call.kind,
        EmbeddedSqlKind::ImmutableLocal
    );
}
