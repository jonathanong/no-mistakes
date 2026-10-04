use super::super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::{Path, PathBuf};

pub(super) fn fixture(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-sql-shape-policy/fixture")
            .join(name),
    )
}

fn config(yaml: &str) -> NoMistakesConfig {
    NoMistakesConfig {
        rules: vec![RuleDef {
            rule: RULE_ID.to_string(),
            scope: Some(RuleScope::Repository),
            options: serde_yaml::from_str(yaml).unwrap(),
            ..Default::default()
        }],
        ..Default::default()
    }
}

const BOTH: &str =
    "sqlInclude: ['sql/**/*.sql']\nbannedShapes: [literal-limit, keyset-only-sweep]\n";

/// (line, target) for every finding.
pub(super) fn found(name: &str, yaml: &str, file: &str) -> Vec<(usize, String)> {
    let root = fixture(name);
    let mut found: Vec<_> = check_with_files(&root, &config(yaml), &[root.join(file)])
        .unwrap()
        .into_iter()
        .map(|finding| (finding.line, finding.target.unwrap()))
        .collect();
    found.sort();
    found
}

pub(super) fn at(findings: &[(usize, &str)]) -> Vec<(usize, String)> {
    findings
        .iter()
        .map(|(line, target)| (*line, target.to_string()))
        .collect()
}

#[test]
fn literal_limits_and_whole_key_walks_are_reported() {
    // The optional-cursor walk is selective until `deleted_at IS NULL` is declared non-selective.
    assert_eq!(
        found("fail-bounded-iteration", BOTH, "sql/001.sql"),
        at(&[
            (1, "keyset-only-sweep"),
            (1, "literal-limit"),
            (3, "keyset-only-sweep"),
            (3, "literal-limit"),
            (4, "keyset-only-sweep"),
            (4, "literal-limit"),
        ])
    );
    let configured = format!(
        "{BOTH}shapeOptions:\n  keysetOnlySweep:\n    nonSelectivePredicates: ['deleted_at IS NULL']\n"
    );
    assert_eq!(
        found("fail-bounded-iteration", &configured, "sql/001.sql"),
        at(&[
            (1, "keyset-only-sweep"),
            (1, "literal-limit"),
            (2, "keyset-only-sweep"),
            (3, "keyset-only-sweep"),
            (3, "literal-limit"),
            (4, "keyset-only-sweep"),
            (4, "literal-limit"),
        ])
    );
}

#[test]
fn work_selecting_statements_and_bound_batch_sizes_are_clean() {
    assert_eq!(found("pass-bounded-iteration", BOTH, "sql/001.sql"), []);
}

#[test]
fn a_lower_and_an_upper_cursor_bound_together_are_a_window() {
    // `id >= $1 AND id < $2` pages a range, not the whole table.
    assert_eq!(found("pass-bounded-iteration", BOTH, "sql/002.sql"), []);
    // One bound alone is still a walk from the cursor to the end.
    assert_eq!(
        found("fail-bounded-iteration", BOTH, "sql/001.sql")
            .iter()
            .filter(|(line, target)| *line == 1 && target == "keyset-only-sweep")
            .count(),
        1
    );
}

#[test]
fn a_seeded_where_selects_nothing_and_a_sampled_table_is_not_walked_whole() {
    // `WHERE TRUE` and `1 = 1` are constants; the TABLESAMPLE restricts the relation first; a
    // window whose bounds are both optional is whole when both binds are NULL; and `ORDER BY
    // random` orders by the computed output column, not a column of the table.
    assert_eq!(
        found("fail-bounded-iteration", BOTH, "sql/003.sql"),
        at(&[
            (1, "keyset-only-sweep"),
            (2, "keyset-only-sweep"),
            (4, "keyset-only-sweep")
        ])
    );
}

#[test]
fn each_shape_is_opt_in_and_independent() {
    let only = |shape: &str| format!("sqlInclude: ['sql/**/*.sql']\nbannedShapes: [{shape}]\n");
    assert_eq!(
        found(
            "fail-bounded-iteration",
            "sqlInclude: ['sql/**/*.sql']",
            "sql/001.sql"
        ),
        []
    );
    let limits = found(
        "fail-bounded-iteration",
        &only("literal-limit"),
        "sql/001.sql",
    );
    assert_eq!(
        limits,
        at(&[
            (1, "literal-limit"),
            (3, "literal-limit"),
            (4, "literal-limit")
        ])
    );
    let sweeps = found(
        "fail-bounded-iteration",
        &only("keyset-only-sweep"),
        "sql/001.sql",
    );
    assert_eq!(
        sweeps,
        at(&[
            (1, "keyset-only-sweep"),
            (3, "keyset-only-sweep"),
            (4, "keyset-only-sweep")
        ])
    );
}

#[test]
fn allowed_values_ignored_tables_and_non_selective_predicates_are_honored() {
    let yaml = format!(
        "{BOTH}shapeOptions:\n  literalLimit:\n    allowedValues: [1, 20]\n  keysetOnlySweep:\n    \
         nonSelectivePredicates: ['deleted_at=FALSE']\n    ignoreTables: [Currencies]\n"
    );
    assert_eq!(
        found("allow-bounded-iteration", &yaml, "sql/001.sql"),
        at(&[
            (5, "keyset-only-sweep"),
            (6, "keyset-only-sweep"),
            (6, "literal-limit")
        ])
    );
    // An empty allowedValues list allows no literal, not even 1.
    let none = format!("{BOTH}shapeOptions:\n  literalLimit:\n    allowedValues: []\n");
    let limits: Vec<_> = found("allow-bounded-iteration", &none, "sql/001.sql")
        .into_iter()
        .filter(|(_, target)| target == "literal-limit")
        .map(|(line, _)| line)
        .collect();
    assert_eq!(limits, [1, 2, 6]);
}

#[test]
fn a_quoted_dot_is_not_a_schema_qualifier_for_ignored_tables_and_an_inner_order_by_is_a_walk() {
    let sweeps = "sqlInclude: ['sql/**/*.sql']\nbannedShapes: [keyset-only-sweep]\n";
    // `(SELECT … ORDER BY id) LIMIT $1` walks the table whole; the one with a selective predicate does not.
    assert_eq!(
        found("quoted-bounded-iteration", sweeps, "sql/001.sql"),
        at(&[
            (1, "keyset-only-sweep"),
            (2, "keyset-only-sweep"),
            (3, "keyset-only-sweep"),
        ])
    );
    // `items` names the table `work.items` by its last part, but not the one table `"work.items"`.
    let ignored = format!("{sweeps}shapeOptions:\n  keysetOnlySweep:\n    ignoreTables: [items]\n");
    assert_eq!(
        found("quoted-bounded-iteration", &ignored, "sql/001.sql"),
        at(&[(2, "keyset-only-sweep"), (3, "keyset-only-sweep")])
    );
}

#[test]
fn executor_sql_is_reported_at_its_source_line() {
    let root = fixture("embedded-bounded-iteration");
    let yaml = "include: ['src/**/*.ts']\nimportSpecifier: '@example/db'\nbannedShapes: [literal-limit, keyset-only-sweep]\n";
    let findings = check_with_files(&root, &config(yaml), &[root.join("src/jobs.ts")]).unwrap();
    let mut found: Vec<_> = findings
        .iter()
        .map(|finding| (finding.line, finding.target.clone().unwrap()))
        .collect();
    found.sort();
    assert_eq!(
        found,
        at(&[(5, "keyset-only-sweep"), (5, "literal-limit")]),
        "{findings:#?}"
    );
}

#[test]
fn template_interpolations_are_binds_for_the_cursor_and_the_batch_size() {
    // `${after}` is the keyset cursor and `${size}` a tunable batch size, like `$1` and `$2`.
    let root = fixture("embedded-bounded-iteration");
    let yaml = "include: ['src/**/*.mts']\nimportSpecifier: '@example/db'\nbannedShapes: [literal-limit, keyset-only-sweep]\n";
    let findings =
        check_with_files(&root, &config(yaml), &[root.join("src/templates.mts")]).unwrap();
    let found: Vec<_> = findings
        .iter()
        .map(|finding| (finding.line, finding.target.clone().unwrap()))
        .collect();
    assert_eq!(found, at(&[(5, "keyset-only-sweep")]), "{findings:#?}");
}

#[test]
fn embedded_marker_identifiers_keep_their_source_provenance() {
    let root = fixture("embedded-bounded-iteration");
    let yaml = "include: ['src/**/*.mts']\nimportSpecifier: '@example/db'\nbannedShapes: [literal-limit, keyset-only-sweep]\n";
    let findings =
        check_with_files(&root, &config(yaml), &[root.join("src/templates.mts")]).unwrap();
    assert_eq!(
        findings
            .iter()
            .map(|finding| (finding.line, finding.target.clone().unwrap()))
            .collect::<Vec<_>>(),
        at(&[(5, "keyset-only-sweep")]),
        "{findings:#?}"
    );
}

#[test]
fn builder_fragments_preserve_interpolation_provenance() {
    let root = fixture("builder-fragment-provenance");
    let yaml = "include: ['src/**/*.mts']\nbannedShapes: [keyset-only-sweep]\n";
    let findings =
        check_with_files(&root, &config(yaml), &[root.join("src/fragments.mts")]).unwrap();
    assert_eq!(
        findings
            .iter()
            .map(|finding| (finding.line, finding.target.clone().unwrap()))
            .collect::<Vec<_>>(),
        at(&[(4, "keyset-only-sweep"), (13, "keyset-only-sweep")]),
        "{findings:#?}"
    );
}

#[test]
fn an_implicit_fetch_is_reported_at_the_fetch_clause() {
    // `FETCH FIRST ROW ONLY` writes no count, so the finding points at the FETCH keyword (a
    // line-level suppression beside it applies), not at the start of the query.
    let none = "sqlInclude: ['sql/**/*.sql']\nbannedShapes: [literal-limit]\nshapeOptions:\n  \
                literalLimit:\n    allowedValues: []\n";
    assert_eq!(
        found("fail-bounded-iteration", none, "sql/002.sql"),
        at(&[(3, "literal-limit"), (6, "literal-limit")])
    );
}

#[test]
fn unanalyzable_sql_names_an_enabled_iteration_shape() {
    let shapes = |literal, sweep| BannedShapes {
        literal_limit: literal,
        keyset_only_sweep: sweep,
        ..Default::default()
    };
    assert_eq!(shapes(true, true).unanalyzable_target(), "literal-limit");
    assert_eq!(
        shapes(false, true).unanalyzable_target(),
        "keyset-only-sweep"
    );
}

#[test]
fn shape_options_are_validated() {
    let error = |yaml: &str| {
        compile_options(&serde_yaml::from_str::<Options>(yaml).unwrap())
            .err()
            .unwrap()
            .to_string()
    };
    let negative = error("shapeOptions: {literalLimit: {allowedValues: [1, -2]}}");
    assert!(
        negative.contains("allowedValues: negative value -2"),
        "{negative}"
    );
    let predicate = error("shapeOptions: {keysetOnlySweep: {nonSelectivePredicates: [' ']}}");
    assert!(
        predicate.contains("nonSelectivePredicates: empty string"),
        "{predicate}"
    );
    let table = error("shapeOptions: {keysetOnlySweep: {ignoreTables: ['']}}");
    assert!(table.contains("ignoreTables: empty string"), "{table}");
    assert!(compile_options(&Options::default()).is_ok());
    // An unparseable predicate is still compared textually.
    assert!(compile_options(
        &serde_yaml::from_str::<Options>(
            "shapeOptions: {keysetOnlySweep: {nonSelectivePredicates: ['(((']}}"
        )
        .unwrap()
    )
    .is_ok());
    let _ = Path::new("");
}
#[test]
fn digit_separator_limits_are_literals() {
    assert_eq!(
        found(
            "digit-separators",
            "sqlInclude: ['sql/**/*.sql']\nbannedShapes: [literal-limit]\n",
            "sql/001.sql"
        ),
        at(&[
            (1, "literal-limit"),
            (2, "literal-limit"),
            (3, "literal-limit")
        ])
    );
}

#[test]
fn ignore_tables_preserves_quoted_case_and_dots() {
    let yaml = "sqlInclude: ['sql/**/*.sql']\nbannedShapes: [keyset-only-sweep]\nshapeOptions:\n  keysetOnlySweep:\n    ignoreTables: ";
    let unquoted_config = format!("{yaml}[Orders]\n");
    assert_eq!(
        found("quoted-table-case", &unquoted_config, "sql/001.sql"),
        at(&[
            (2, "keyset-only-sweep"),
            (5, "keyset-only-sweep"),
            (6, "keyset-only-sweep"),
        ])
    );
    let root = fixture("quoted-table-case");
    let findings = check_with_files(
        &root,
        &config(&unquoted_config),
        &[root.join("sql/001.sql")],
    )
    .unwrap();
    assert!(findings
        .iter()
        .find(|finding| finding.line == 2)
        .unwrap()
        .message
        .contains("list `\"Orders\"` in"));
    assert!(findings
        .iter()
        .find(|finding| finding.line == 5)
        .unwrap()
        .message
        .contains("list `\"work.Orders\"` in"));
    assert!(findings
        .iter()
        .find(|finding| finding.line == 6)
        .unwrap()
        .message
        .contains("list `\" orders \"` in"));
    assert_eq!(
        found(
            "quoted-table-case",
            &format!("{yaml}['\"Orders\"', '\"work.Orders\"']\n"),
            "sql/001.sql"
        ),
        at(&[
            (3, "keyset-only-sweep"),
            (4, "keyset-only-sweep"),
            (6, "keyset-only-sweep"),
        ])
    );
}

#[test]
fn radix_prefix_separators_preserve_literal_limit_findings() {
    assert_eq!(
        found(
            "radix-prefix-separators",
            "sqlInclude: ['sql/**/*.sql']\nbannedShapes: [literal-limit]\nshapeOptions:\n  literalLimit:\n    allowedValues: []\n",
            "sql/pages.sql"
        ),
        at(&[
            (1, "literal-limit"),
            (2, "literal-limit"),
            (3, "literal-limit")
        ])
    );
}

#[test]
fn numeric_hex_literal_limits_are_reported_and_suppressible() {
    let root = fixture("radix-hex-literals");
    let file = root.join("sql/pages.sql");
    let yaml = "sqlInclude: ['sql/**/*.sql']\nbannedShapes: [literal-limit]\nshapeOptions:\n  literalLimit:\n    allowedValues: []\n";
    let mut findings = check_with_files(&root, &config(yaml), std::slice::from_ref(&file)).unwrap();
    let sources = crate::codebase::rules::source_store_for_files(std::slice::from_ref(&file));
    crate::codebase::rules::suppress_rule_findings_with_sources(&root, &mut findings, &sources);
    assert_eq!(
        findings
            .into_iter()
            .map(|finding| (finding.line, finding.target.unwrap()))
            .collect::<Vec<_>>(),
        at(&[(1, "literal-limit"), (6, "literal-limit")])
    );
}

#[test]
fn expanded_keysets_follow_the_order_keys_before_the_rule_reports_a_walk() {
    let config = "sqlInclude: ['sql/**/*.sql']\nbannedShapes: [keyset-only-sweep]\n";
    assert_eq!(
        found("expanded-keysets", config, "sql/pages.sql"),
        at(&[
            (2, "keyset-only-sweep"),
            (3, "keyset-only-sweep"),
            (10, "keyset-only-sweep"),
        ])
    );
}

#[test]
fn reordered_expanded_keysets_keep_the_same_rule_findings() {
    let config = "sqlInclude: ['sql/**/*.sql']\nbannedShapes: [keyset-only-sweep]\n";
    assert_eq!(
        found("reordered-expanded-keysets", config, "sql/pages.sql"),
        at(&[(2, "keyset-only-sweep"), (3, "keyset-only-sweep")])
    );
}

#[test]
fn commuted_expanded_keysets_keep_the_same_rule_findings() {
    let config = "sqlInclude: ['sql/**/*.sql']\nbannedShapes: [keyset-only-sweep]\n";
    assert_eq!(
        found("commuted-expanded-keysets", config, "sql/pages.sql"),
        at(&[(2, "keyset-only-sweep"), (3, "keyset-only-sweep")])
    );
}

#[test]
fn casted_expanded_keysets_preserve_bind_identity_in_rule_findings() {
    let config = "sqlInclude: ['sql/**/*.sql']\nbannedShapes: [keyset-only-sweep]\n";
    assert_eq!(
        found("casted-expanded-keysets", config, "sql/pages.sql"),
        at(&[
            (2, "keyset-only-sweep"),
            (8, "keyset-only-sweep"),
            (9, "keyset-only-sweep"),
            (15, "keyset-only-sweep"),
            (19, "keyset-only-sweep"),
            (21, "keyset-only-sweep"),
            (23, "keyset-only-sweep"),
        ])
    );
}

#[test]
fn non_selective_predicates_preserve_literal_case() {
    let yaml = "sqlInclude: ['sql/**/*.sql']\nbannedShapes: [keyset-only-sweep]\nshapeOptions:\n  keysetOnlySweep:\n    nonSelectivePredicates: [\"status = 'idle'\"]\n";
    assert_eq!(
        found("predicate-case", yaml, "sql/001.sql"),
        at(&[(2, "keyset-only-sweep")])
    );
}

#[test]
fn bind_only_guard_does_not_hide_a_sweep() {
    assert_eq!(
        found(
            "bind-guards",
            "sqlInclude: ['sql/**/*.sql']\nbannedShapes: [keyset-only-sweep]\n",
            "sql/001.sql"
        ),
        at(&[(1, "keyset-only-sweep"), (4, "keyset-only-sweep"),])
    );
}
