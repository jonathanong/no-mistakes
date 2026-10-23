use super::support::fixture;

#[test]
fn append_preserves_missing_proofs_and_invalidates_eof_only_classifications() {
    let (source, _, heads) = fixture("append-missing.mts");
    let mut count = 0;
    let mut offset = 0;
    for line in source.split_inclusive('\n') {
        if let Some((before, expected)) = line.split_once("// head: ") {
            let start = offset + before.find("read(").unwrap();
            let (&span, _) = heads
                .calls
                .iter()
                .find(|(span, _)| span.0 as usize == start)
                .unwrap();
            let observed = if heads.accepts(span) {
                "clear"
            } else if heads.proves_missing(span) {
                "missing"
            } else {
                "unknown"
            };
            assert_eq!(observed, expected.trim(), "{line}");
            count += 1;
        }
        offset += line.len();
    }
    assert_eq!(count, 34);
}
