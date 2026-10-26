#[test]
fn missing_prepared_file_cannot_prove_a_loop_backedge() {
    let root = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("loop-carried-reassignment");
    let options =
        super::super::super::compile_options(&super::super::super::Options::default()).unwrap();
    assert!(!super::proves_missing(
        &crate::codebase::check_facts::CheckFactMap::default(),
        &root.join("src/query.mts"),
        &options,
        (0, 0),
    ));
}
