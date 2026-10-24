use super::*;

#[test]
fn low_nested_grammar_does_no_recursive_schema_or_comparison_work() {
    let files = [root().join("schemas.json")];
    let sources = super::super::super::source_store_for_files(&files);
    let allowed = BTreeSet::from(["schemas.json".into()]);
    let documents = document::Documents::default();
    // Call directly on this thread: Rayon workers do not inherit thread-local
    // probes, and a zero measured on the caller would not prove skipped work.
    for name in ["nested", "objectArray", "arrayObject", "arrayArray"] {
        let contract = contract(name, name);
        let before = probe::counts();
        assert!(matches!(
            low::check(&root(), &contract, &allowed, &sources, &documents).unwrap(),
            low::Outcome::Unanalyzable(_)
        ));
        assert_eq!(probe::counts(), before);
        check_contract(&root(), &contract, &allowed, &sources, &documents).unwrap();
        let after = probe::counts();
        assert!(after.0 > before.0);
        assert!(after.1 > before.1);
        assert!(after.2 > before.2);
    }
    assert_eq!(sources.physical_read_count(), 1);
    assert_eq!(
        documents.parses.load(std::sync::atomic::Ordering::Relaxed),
        1
    );
}

#[test]
fn accepted_low_grammar_still_runs_the_existing_schema_and_subset_proof() {
    let files = [root().join("schemas.json")];
    let sources = super::super::super::source_store_for_files(&files);
    let documents = document::Documents::default();
    let allowed = BTreeSet::from(["schemas.json".into()]);
    for name in ["scalar", "flat", "array"] {
        let before = probe::counts();
        assert!(matches!(
            low::check(
                &root(),
                &contract(name, name),
                &allowed,
                &sources,
                &documents
            )
            .unwrap(),
            low::Outcome::Proven
        ));
        let after = probe::counts();
        assert!(after.0 > before.0);
        assert_eq!(after.1, before.1);
        assert!(after.2 > before.2);
    }
}
