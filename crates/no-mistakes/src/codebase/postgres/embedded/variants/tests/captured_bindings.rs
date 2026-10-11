use super::*;

#[test]
fn delayed_function_execution_does_not_recover_mutable_captured_sql_snapshots() {
    let file = file_facts("variants-captured-bindings.ts");
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
        match expectation {
            "captured opaque" => {
                assert!(call.is_unanalyzable(), "{call:#?}");
                assert_eq!(call.sql_text, None, "{call:#?}");
            }
            "finite" => assert_eq!(call.variants.len(), 2, "{call:#?}"),
            "finite three" => assert_eq!(call.variants.len(), 3, "{call:#?}"),
            "unchanged builder" => {
                assert_eq!(call.kind, EmbeddedSqlKind::ImmutableLocal, "{call:#?}");
                assert_eq!(
                    call.sql_text.as_deref(),
                    Some("SELECT id FROM legacy_users")
                );
                assert!(call.variants.is_empty(), "{call:#?}");
            }
            _ => panic!("unexpected fixture expectation: {expectation}"),
        }
    }
}
