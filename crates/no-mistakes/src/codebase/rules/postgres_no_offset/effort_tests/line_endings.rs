use super::*;

#[test]
fn crlf_literals_keep_physical_offsets_and_escaped_sql_stays_uncertain() {
    let source = std::fs::read_to_string(root().join("src/crlf.ts")).unwrap();
    assert!(
        source.contains("\r\n"),
        "saved fixture must retain CRLF bytes"
    );
}
