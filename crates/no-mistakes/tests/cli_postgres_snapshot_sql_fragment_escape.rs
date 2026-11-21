use super::check_json_snapshot::{assert_saved_fixture, assert_saved_fixture_report};
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
    let cases = [
        ("append_chain", root, config.clone()),
        ("edges", edges, config),
    ];
    assert_saved_fixture("postgres_sql_fragment_escape_append_chain", &cases);
}

#[test]
fn postgres_sql_fragment_escape_sibling_graph_snapshot() {
    let root = no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-sql-fragment-escape/fixtures/sibling-graph"),
    );
    let reports = assert_saved_fixture_report(
        "postgres_sql_fragment_escape_sibling_graph",
        &[("sibling_graph", root.clone(), root.join(".no-mistakes.yml"))],
        &[],
        &[],
    );
    let rules = reports["sibling_graph"]["report"]["rules"]
        .as_array()
        .expect("rules");
    assert!(
        rules.iter().any(|finding| {
            finding["rule"] == "forbidden-calls" && finding["file"] == "place-order.ts"
        }),
        "{rules:?}"
    );
    assert!(
        rules.iter().any(|finding| {
            finding["rule"] == "postgres-sql-fragment-escape"
                && finding["file"] == "extend.ts"
                && finding["sourceLine"].is_string()
        }),
        "{rules:?}"
    );
    assert!(
        rules.iter().any(|finding| {
            finding["rule"] == "unconstructed-error-class" && finding["file"] == "orphan-error.ts"
        }),
        "{rules:?}"
    );
    assert!(
        rules.iter().all(|finding| {
            finding["message"]
                .as_str()
                .is_none_or(|message| !message.contains("WidgetError"))
        }),
        "{rules:?}"
    );
    for finding in rules {
        if finding.get("file").and_then(|file| file.as_str()).is_some() {
            assert!(finding["sourceLine"].is_string(), "{finding}");
        }
    }
}
