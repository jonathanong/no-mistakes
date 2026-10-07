#[test]
fn incomplete_escape_sequence_never_becomes_literal_text() {
    let source = crate::codebase::postgres::source::tests::fixture("truncated-escape-value.txt");
    assert!(super::decode(source.trim_end()).is_none());
}
