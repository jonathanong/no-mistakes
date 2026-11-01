use super::check_json_snapshot::{assert_saved_fixture, rules_path};
use std::path::PathBuf;

fn repo_rel(rel: &str) -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(rel),
    )
}

#[test]
fn postgres_bounded_statements_chains_check_json_snapshot() {
    let root = "test-cases/rules/postgres-bounded-statements/fixture/bounded-chains";
    let reuse = "test-cases/rules/postgres-bounded-statements/fixture/bounded-reuse";
    let cases = [
        // LIMIT 1 keeps the later primary-key writes quiet; the open chain still reports
        ("capped_cte_chain", root, "bounded-chains-capped.yml"),
        // await using + sql`LIMIT ${batchSize}` stays quiet; the uncapped delete reports
        ("array_subquery_delete", root, "bounded-chains-delete.yml"),
        // caller-sized recordset and unnest stay quiet; unbounded inputs still report
        ("recordset_and_unnest", root, "bounded-chains-record.yml"),
        // without columnBounds the union still reports every arm
        (
            "union_without_column_bounds",
            root,
            "bounded-chains-union.yml",
        ),
        // columnBounds on each filtered base column, including FOR UPDATE OF p
        (
            "union_column_bounds",
            root,
            "bounded-chains-union-columns.yml",
        ),
        // #2979 shape (1): fn_remaining() in the capped CTE may return a set, so the cap is
        // not trusted; the builtin projection keeps it
        ("reuse_sibling_function", reuse, "bounded-reuse-sibling.yml"),
        ("reuse_shape1_function", reuse, "bounded-reuse-shape1.yml"),
        ("reuse_shape1_builtin", reuse, "bounded-reuse-shape1-builtin.yml"),
        ("reuse_shape1_no_limit", reuse, "bounded-reuse-shape1-nolimit.yml"),
        ("reuse_shape1_unpinned", reuse, "bounded-reuse-shape1-unpinned.yml"),
    ]
    .map(|(label, root, config)| {
        (
            label,
            repo_rel(root),
            rules_path("postgres-bounded-statements", "configs", config),
        )
    });
    assert_saved_fixture("postgres_bounded_statements_chains", &cases);
}
