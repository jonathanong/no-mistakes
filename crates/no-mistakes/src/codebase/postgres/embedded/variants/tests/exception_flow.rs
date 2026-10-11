use super::*;

#[test]
fn handlers_and_finalizers_never_read_only_the_preceding_phases_end_snapshot() {
    let file = file_facts("variants-exception-flow.ts");
    let source = std::fs::read_to_string(&file.path).unwrap();
    let expected = source
        .lines()
        .enumerate()
        .filter_map(|(index, line)| {
            line.split_once("// ")
                .map(|(_, expectation)| (index + 1, expectation))
        })
        .collect::<Vec<_>>();
    assert_eq!(file.calls.len(), expected.len());
    for (call, (line, expectation)) in file.calls.iter().zip(expected) {
        assert_eq!(call.line, line as u32);
        match expectation {
            "opaque" => {
                assert!(call.is_unanalyzable(), "{call:#?}");
                assert_eq!(call.sql_text, None);
            }
            "opaque prefix" => {
                assert!(call.is_unanalyzable(), "{call:#?}");
                assert!(call.sql_text.is_some());
            }
            "finite" => assert_eq!(call.variants.len(), 2, "{call:#?}"),
            "unchanged" => {
                assert_eq!(call.kind, EmbeddedSqlKind::ImmutableLocal);
                assert_eq!(call.sql_text.as_deref(), Some("SELECT 18"));
                assert!(call.variants.is_empty());
            }
            "legacy builder" => {
                assert_eq!(call.kind, EmbeddedSqlKind::ImmutableLocal);
                assert_eq!(call.sql_text.as_deref(), Some("SELECT 25"));
                assert!(call.variants.is_empty());
            }
            "inline" => {
                assert_eq!(call.kind, EmbeddedSqlKind::Inline);
                assert_eq!(call.sql_text.as_deref(), Some("SELECT 19"));
                assert!(call.variants.is_empty());
            }
            _ => panic!("unknown expectation {expectation}"),
        }
    }
}
