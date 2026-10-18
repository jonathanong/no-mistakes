use super::*;

#[test]
fn destructuring_writes_invalidate_builders_but_preserve_independent_receivers() {
    let file = file_facts("variants-destructuring-assignments.ts");
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
    let mut actual_lines = file.calls.iter().map(|call| call.line).collect::<Vec<_>>();
    actual_lines.sort_unstable();
    assert_eq!(
        actual_lines,
        expected.iter().map(|(line, _)| *line).collect::<Vec<_>>()
    );
    for call in &file.calls {
        let expectation = expected
            .iter()
            .find(|(line, _)| *line == call.line)
            .unwrap()
            .1;
        if expectation == "opaque" || expectation == "opaque placeholders" {
            assert!(call.is_unanalyzable(), "{call:#?}");
            assert!(call.sql_source_positions.is_empty(), "{call:#?}");
            if expectation == "opaque placeholders" {
                assert_eq!(
                    call.sql_text.as_deref(),
                    Some("sql_placeholder_1 sql_placeholder_2")
                );
                assert_eq!(call.recovered_placeholder_positions, [(1, 1), (1, 19)]);
            } else {
                assert_eq!(call.sql_text, None, "{call:#?}");
                assert!(call.recovered_placeholder_positions.is_empty(), "{call:#?}");
            }
        } else {
            let mut expected = expectation
                .strip_prefix("variants: ")
                .unwrap()
                .split(" | ")
                .collect::<Vec<_>>();
            let mut actual = texts(call);
            actual.sort_unstable();
            expected.sort_unstable();
            assert_eq!(actual, expected, "{call:#?}");
        }
    }
    assert_eq!(file.calls, calls("variants-destructuring-assignments.ts"));
}
