use super::support::{
    assert_markers, decisions, fixture_with_append_candidates, fixture_with_effects,
};

#[test]
fn builder_write_census_rejects_non_append_mutations_before_and_after_reads() {
    let (source, _, heads) = fixture_with_effects("builder-write-census.mts");
    assert_eq!(assert_markers(&source, &heads), 19);
}

#[test]
fn builder_write_census_legacy_candidates_reject_unsafe_builder_lifetimes() {
    let (source, _, heads) = fixture_with_append_candidates("builder-write-census.mts", true);
    let safe_start = source.find("function appendAfterRead(").unwrap();
    let safe_end = safe_start + source[safe_start..].find("\n}").unwrap();
    let mut safe = 0;
    for (span, _) in decisions(&source, &heads) {
        let start = span.0 as usize;
        let line_start = source[..start].rfind('\n').map_or(0, |index| index + 1);
        let line = source[line_start..].lines().next().unwrap();
        if line.contains("// low: false") {
            assert!(!heads.accepts_trusted_append(span), "{line}");
        }
        if (safe_start..safe_end).contains(&start) {
            assert!(heads.accepts_trusted_append(span), "{line}");
            safe += 1;
        }
    }
    assert_eq!(safe, 2);
}
