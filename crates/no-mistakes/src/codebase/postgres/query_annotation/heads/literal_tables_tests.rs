use super::support::{assert_markers, fixture};

#[test]
fn closed_literal_tables_require_only_static_executor_reads() {
    let (source, _, heads) = fixture("literal-tables.mts");
    assert_eq!(assert_markers(&source, &heads), 28);
}

#[test]
fn a_table_alone_requests_the_binding_gap_index() {
    let (source, _, heads) = fixture("literal-table-only.mts");
    assert_eq!(assert_markers(&source, &heads), 1);
}

#[test]
fn dynamic_lexical_access_keeps_tables_unproven() {
    let (source, _, heads) = fixture("literal-table-gap.mts");
    assert_eq!(assert_markers(&source, &heads), 1);
}

#[test]
fn literal_tables_bound_depth_and_leaf_count() {
    let (source, _, heads) = fixture("literal-table-bounds.mts");
    assert_eq!(assert_markers(&source, &heads), 4);
}

#[test]
fn dynamic_access_inside_a_reader_keeps_the_table_unproven() {
    let (source, _, heads) = fixture("literal-table-ref-gap.mts");
    assert_eq!(assert_markers(&source, &heads), 1);
}

#[test]
fn unavailable_module_bindings_keep_tables_unproven() {
    let (_, mut ts, _) = fixture("literal-table-only.mts");
    ts.module_bindings = None;
    let heads = super::support::recollect("literal-table-only.mts", &ts);
    assert!(heads
        .calls
        .values()
        .all(|proof| *proof == super::super::HeadProof::Unproven));
}
