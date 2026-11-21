use super::check_json_snapshot::assert_saved_fixture;
use std::path::PathBuf;

#[test]
fn postgres_sql_fragment_escape_append_chain_snapshot() {
    let root = no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/eslint-snippets/fixture/executor-append-chain"),
    );
    let config = no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-sql-fragment-escape/configs/append-chain.yml"),
    );
    // The fixture tree is the existing append-chain parity set. Findings are
    // the escape site, and files that only append a fragment stay absent.
    let edges = no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-sql-fragment-escape/fixtures/edges"),
    );
    let cases = [("append_chain", root, config.clone()), ("edges", edges, config)];
    assert_saved_fixture("postgres_sql_fragment_escape_append_chain", &cases);
}
