use super::*;

#[test]
fn immutable_guard_aliases_share_truth_and_mutable_copies_keep_independent_snapshots() {
    let file = file_facts("variants-guard-alias.ts");
    let source = std::fs::read_to_string(&file.path).unwrap();
    let expected = source
        .lines()
        .enumerate()
        .filter_map(|(index, line)| {
            line.split_once("// variants: ")
                .map(|(_, sql)| (index as u32 + 1, sql.split(" | ").collect::<Vec<_>>()))
        })
        .collect::<Vec<_>>();
    assert_eq!(file.calls.len(), expected.len(), "{:#?}", file.calls);
    for (call, (line, mut expected)) in file.calls.iter().zip(expected) {
        assert_eq!(call.line, line, "{call:#?}");
        let mut actual = texts(call);
        actual.sort_unstable();
        expected.sort_unstable();
        assert_eq!(actual, expected, "{call:#?}");
        assert_eq!(call.kind, EmbeddedSqlKind::Dynamic);
    }
    assert_eq!(file.calls, calls("variants-guard-alias.ts"));
}
