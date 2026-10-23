use super::*;

#[test]
fn crlf_literals_keep_physical_offsets_and_escaped_sql_stays_uncertain() {
    let source = std::fs::read_to_string(root().join("src/crlf.ts")).unwrap();
    assert!(
        source.contains("\r\n"),
        "saved fixture must retain CRLF bytes"
    );
    let max = run(".no-mistakes.crlf-max.yml");
    let low = run(".no-mistakes.crlf-low.yml");
    let ignore = run(".no-mistakes.crlf-ignore.yml");
    assert_eq!(max.len(), 3, "{max:#?}");
    assert_eq!(low.len(), 3, "{low:#?}");
    assert_eq!(ignore.len(), 2, "{ignore:#?}");
    assert!(ignore.iter().all(|finding| max.contains(finding)));
    assert_eq!(
        ignore
            .iter()
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        [4, 8]
    );
    // Physical CRLF is accepted; a JavaScript escape within CRLF SQL still abstains.
    assert_eq!(
        low.iter()
            .filter(|finding| finding.target.as_deref() == Some("unanalyzable"))
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        [10]
    );
}
