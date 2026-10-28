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
