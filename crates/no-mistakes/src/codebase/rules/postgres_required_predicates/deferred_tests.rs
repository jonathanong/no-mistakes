// Output cases moved to cli_postgres_snapshot_required_predicates.
use super::columns_tests::fixture;
use super::*;

const REQUIRED: &str =
    "sqlInclude: [sql/**/*.sql]\nrelations: [{table: orders, requireColumns: [account_id]}]\n";

#[test]
fn prepared_scans_reject_an_absent_request() {
    let root = fixture("deferred");
    let file = root.join("sql/self-in.sql");
    let sources = super::super::source_store_for_files(std::slice::from_ref(&file));
    let opts = compile_options(&serde_yaml::from_str(REQUIRED).unwrap()).unwrap();
    let error = scan::scan(&root, &opts, &[file], &sources, None).unwrap_err();
    assert!(error
        .to_string()
        .contains("prepared PostgreSQL facts are required"));
}
