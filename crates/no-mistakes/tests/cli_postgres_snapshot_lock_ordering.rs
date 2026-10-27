use super::check_json_snapshot::{assert_saved_fixture, rules_path};
use std::path::PathBuf;

fn repo_rel(rel: &str) -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(rel),
    )
}

fn repo_case(name: &str) -> PathBuf {
    repo_rel(&format!("fixtures/postgres/lock-ordering/{name}"))
}

fn fix(name: &str) -> PathBuf {
    rules_path("postgres-lock-ordering", "fixture", name)
}

fn push(
    cases: &mut Vec<(&'static str, PathBuf, PathBuf)>,
    config: &PathBuf,
    items: &[(&'static str, PathBuf)],
) {
    for &(label, ref root) in items {
        cases.push((label, PathBuf::clone(root), PathBuf::clone(config)));
    }
}

#[test]
fn postgres_lock_ordering_check_json_snapshot() {
    let rule = "postgres-lock-ordering";
    let default = rules_path(rule, "configs", "default.yml");
    let catalog = rules_path(rule, "configs", "catalog.yml");
    let mut cases = Vec::new();
    push(
        &mut cases,
        &default,
        &[
            // src/disabled.ts is suppressed and must not add a finding.
            ("fail_abba", fix("fail")),
            ("pass_order", fix("pass-order")),
            ("pass_skip", fix("pass-skip")),
            ("pass_directive", fix("pass-directive")),
            ("unparseable", fix("unparseable")),
            ("pass_multi_target", fix("pass-multi-target")),
            ("fail_multi_target", fix("fail-multi-target")),
            ("pass_key_strengths", fix("pass-key-strengths")),
            ("fail_key_strengths", fix("fail-key-strengths")),
            (
                "fail_unique_key_no_catalog",
                fix("fail-unique-key-no-catalog"),
            ),
            ("pass_sql_comment", fix("pass-sql-comment")),
            ("too_far", fix("too-far")),
            ("no_lock", fix("no-lock")),
            // Non-locking dynamic read, the safe-directive call, and the suppressed call stay quiet.
            ("unanalyzable", fix("unanalyzable")),
            ("disabled_comment", fix("disabled-comment")),
            // Target must not be "unparseable".
            ("parsed_equality_lock", fix("parsed-equality-lock")),
            ("unparseable_open", fix("unparseable-open")),
            // Target must not contain "unanalyzable".
            ("one_bad", rules_path(rule, "variants", "one-bad")),
        ],
    );
    push(
        &mut cases,
        &catalog,
        &[
            ("pass_catalog", repo_case("pass-catalog")),
            ("pass_catalog_of_alias", repo_case("pass-catalog-of-alias")),
            (
                "pass_catalog_unqualified",
                repo_case("pass-catalog-unqualified"),
            ),
            ("fail_catalog_order", repo_case("fail-catalog-order")),
            ("fail_catalog_join", repo_case("fail-catalog-join")),
            (
                "fail_catalog_qualified_other",
                repo_case("fail-catalog-qualified-other"),
            ),
            (
                "fail_catalog_schema_qualified",
                repo_case("fail-catalog-schema-qualified"),
            ),
            (
                "pass_pinned_key_prefix",
                repo_case("pass-pinned-key-prefix"),
            ),
            (
                "fail_pinned_key_prefix",
                repo_case("fail-pinned-key-prefix"),
            ),
            ("pass_join_pinned", fix("pass-join-pinned")),
            ("fail_join_pinned", fix("fail-join-pinned")),
            (
                "pass_catalog_multi_target",
                fix("pass-catalog-multi-target"),
            ),
            (
                "fail_catalog_multi_target",
                fix("fail-catalog-multi-target"),
            ),
            (
                "fail_catalog_multi_target_unresolved",
                fix("fail-catalog-multi-target-unresolved"),
            ),
            ("pass_unique_key_filter", fix("pass-unique-key-filter")),
            ("fail_unique_key_partial", fix("fail-unique-key-partial")),
            ("pass_catalog_lateral", fix("pass-catalog-lateral")),
            ("fail_catalog_lateral", fix("fail-catalog-lateral")),
            // Messages must not contain "unique-key order".
            (
                "fail_catalog_interpolated",
                fix("fail-catalog-interpolated"),
            ),
        ],
    );
    push(
        &mut cases,
        &rules_path(rule, "configs", "exclude-lock.yml"),
        &[("exclude_lock", fix("fail"))],
    );
    push(
        &mut cases,
        &rules_path(rule, "configs", "include-lock.yml"),
        &[("include_lock", fix("fail"))],
    );
    push(
        &mut cases,
        &rules_path(rule, "configs", "include-missing.yml"),
        &[("include_missing", fix("fail"))],
    );
    push(
        &mut cases,
        &rules_path(rule, "configs", "other-executor.yml"),
        &[("other_executor", fix("fail"))],
    );
    push(
        &mut cases,
        &rules_path(rule, "configs", "ordered-locks.yml"),
        &[("ordered_locks", fix("custom-directive"))],
    );
    push(
        &mut cases,
        &rules_path(rule, "configs", "ignore.yml"),
        &[("unanalyzable_ignored", fix("unanalyzable"))],
    );
    assert_saved_fixture("postgres_lock_ordering", &cases);
}
