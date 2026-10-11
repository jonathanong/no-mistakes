use super::*;

#[test]
fn switch_member_invocations_detect_writes_and_preserve_inert_callbacks() {
    let file = file_facts("variants-switch-member-invocations.ts");
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
        if expectation == "opaque" {
            assert!(call.is_unanalyzable(), "{call:#?}");
            assert_eq!(call.sql_text, None, "{call:#?}");
        } else if let Some(sql) = expectation.strip_prefix("finite: ") {
            assert_eq!(texts(call), [sql], "{call:#?}");
        } else {
            let (kind, sql) = expectation.split_once(": ").unwrap();
            assert_eq!(
                call.kind,
                if kind == "inline" {
                    EmbeddedSqlKind::Inline
                } else {
                    EmbeddedSqlKind::ImmutableLocal
                },
                "{call:#?}"
            );
            assert_eq!(call.sql_text.as_deref(), Some(sql), "{call:#?}");
            assert!(call.variants.is_empty(), "{call:#?}");
        }
    }
    assert_eq!(file.calls, calls("variants-switch-member-invocations.ts"));
}
