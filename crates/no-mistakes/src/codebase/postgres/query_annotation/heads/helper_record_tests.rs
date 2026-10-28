use super::support::{assert_markers, fixture};

#[test]
fn fresh_helper_records_need_exact_paired_reads_and_stable_source() {
    let (source, _, heads) = fixture("helper-records.mts");
    assert_eq!(assert_markers(&source, &heads), 19);
    let missing = source.find("read(absent.text").unwrap() as u32;
    let unmarked = source.find("read(unmarked.text").unwrap() as u32;
    for start in [missing, unmarked] {
        assert_eq!(
            heads
                .calls
                .iter()
                .find(|(span, _)| span.0 == start)
                .map(|(_, proof)| *proof),
            Some(super::super::HeadProof::Missing),
        );
    }
}

#[test]
fn immutable_record_text_and_exact_local_destructuring() {
    let (source, _, heads) = fixture("record-text.mts");
    assert_eq!(assert_markers(&source, &heads), 9);
    let missing = source.find("read(missing.text").unwrap() as u32;
    let call = heads
        .calls
        .iter()
        .find(|(span, _)| span.0 == missing)
        .unwrap();
    assert_eq!(*call.1, super::super::HeadProof::Missing);
    let missing = source.find("read(text, values) // low: false").unwrap() as u32;
    let call = heads
        .calls
        .iter()
        .find(|(span, _)| span.0 == missing)
        .unwrap();
    assert_eq!(*call.1, super::super::HeadProof::Missing);
}
