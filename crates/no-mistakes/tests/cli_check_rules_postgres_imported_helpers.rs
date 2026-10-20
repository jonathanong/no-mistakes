//! Executor SQL recovered from imported helpers and literal-identifier templates.
//! Every scenario is a saved fixture under `test-cases/rules/<rule>/variants/`.

use std::path::PathBuf;
use std::process::Command;

const REQUIRED: &str = "postgres-required-predicates";
const VIEW: &str = "view_active_plans";
const UNANALYZABLE: &str = "unanalyzable";

/// `(file, line, target)` of every finding, sorted.
type Findings = Vec<(String, u64, String)>;

fn run(rule: &str, name: &str, include_suppressed: bool) -> serde_json::Value {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules")
        .join(rule)
        .join("variants")
        .join(name);
    let mut command = Command::new(env!("CARGO_BIN_EXE_no-mistakes"));
    command
        .args(["check", "--format", "json", "--root"])
        .arg(&root)
        .arg("--config")
        .arg(root.join(".no-mistakes.yml"));
    if include_suppressed {
        command.arg("--include-suppressed");
    }
    let output = command.output().unwrap();
    serde_json::from_slice(&output.stdout)
        .unwrap_or_else(|error| panic!("{error}: {}", String::from_utf8_lossy(&output.stderr)))
}

fn findings_of(rule: &str, name: &str) -> Findings {
    let report = run(rule, name, false);
    let mut findings: Findings = report["rules"]
        .as_array()
        .unwrap()
        .iter()
        .map(|finding| {
            (
                finding["file"].as_str().unwrap().to_string(),
                finding["line"].as_u64().unwrap(),
                finding["target"].as_str().unwrap().to_string(),
            )
        })
        .collect();
    findings.sort();
    findings
}

fn required(name: &str) -> Findings {
    findings_of(REQUIRED, name)
}

fn expect(rows: &[(&str, u64, &str)]) -> Findings {
    let mut rows: Findings = rows
        .iter()
        .map(|(file, line, target)| (file.to_string(), *line, target.to_string()))
        .collect();
    rows.sort();
    rows
}

fn unanalyzable(lines: &[u64]) -> Findings {
    let rows: Vec<_> = lines
        .iter()
        .map(|line| ("src/caller.mts", *line, UNANALYZABLE))
        .collect();
    expect(&rows)
}

#[test]
fn imported_string_helper_with_literal_identifiers_is_recovered() {
    // The finding sits on the helper's own `FROM view_active_plans` line.
    assert_eq!(
        required("imported-literal-identifiers"),
        expect(&[("src/visibility-sql.mts", 3, VIEW)])
    );
}

#[test]
fn builder_with_imported_fragment_helper_and_cte_is_recovered() {
    assert_eq!(
        required("imported-builder-fragment"),
        expect(&[
            ("src/caller.mts", 10, VIEW),
            ("src/eligibility.mts", 5, VIEW)
        ])
    );
}

#[test]
fn tsconfig_paths_alias_imports_are_resolved() {
    assert_eq!(
        required("imported-paths-alias"),
        expect(&[("src/modules/feed-query-builders.mts", 2, VIEW)])
    );
}

#[test]
fn barrel_re_exports_are_followed_across_hops() {
    assert_eq!(
        required("imported-barrel"),
        expect(&[
            ("src/visibility/owner-sql.mts", 2, VIEW),
            ("src/visibility/tier-sql.mts", 2, VIEW),
        ])
    );
}

#[test]
fn every_re_export_form_reaches_its_declaration() {
    assert_eq!(
        required("imported-reexport-forms"),
        expect(&[
            ("src/declares.mts", 2, VIEW),
            ("src/default-export.mts", 2, VIEW),
            ("src/diamond.mts", 2, VIEW),
            // The explicit re-export wins over `shadowed-star.mts`.
            ("src/explicit.mts", 2, VIEW),
        ])
    );
}

#[test]
fn statement_builder_helpers_with_explicit_mts_extensions_are_recovered() {
    assert_eq!(
        required("imported-statement-builder"),
        expect(&[
            ("src/caller.mts", 8, VIEW),
            ("src/caller.mts", 13, VIEW),
            ("src/tier-filter.mts", 5, VIEW),
        ])
    );
}

#[test]
fn relative_scoped_executors_consume_imported_helpers() {
    // `fail` mode: no unanalyzable finding means both scoped executors resolved.
    assert_eq!(
        required("imported-relative-scoped-executor"),
        expect(&[("packages/db/src/orders/visibility-sql.mts", 2, VIEW)])
    );
}

#[test]
fn same_file_helpers_splice_literal_identifiers_too() {
    assert_eq!(
        required("same-file-literal-identifiers"),
        expect(&[
            ("src/caller.mts", 5, VIEW),
            ("src/caller.mts", 9, VIEW),
            // An optional call and a non-literal argument stay unanalyzable.
            // A no-substitution template splices, so it is not a third one.
            ("src/caller.mts", 28, UNANALYZABLE),
            ("src/caller.mts", 35, UNANALYZABLE),
        ])
    );
}

#[test]
fn one_export_shared_by_many_calls_is_reported_once() {
    assert_eq!(
        required("imported-shared-export"),
        expect(&[("src/visibility-sql.mts", 2, VIEW)])
    );
}

#[test]
fn a_directive_in_the_helper_module_suppresses_its_own_findings() {
    assert_eq!(
        required("imported-suppressed"),
        expect(&[("src/visibility-sql.mts", 7, VIEW)])
    );
    let report = run(REQUIRED, "imported-suppressed", true);
    let suppressed = report["suppressed"].as_array().unwrap();
    assert_eq!(suppressed.len(), 1, "{report}");
    assert_eq!(suppressed[0]["file"], "src/visibility-sql.mts");
    assert_eq!(suppressed[0]["line"], 3);
    assert_eq!(suppressed[0]["directive"]["line"], 2);
}

#[test]
fn a_package_alias_wins_over_the_roots_same_named_alias() {
    // The decoy at the root declares line 6; the package's own helper is line 2.
    assert_eq!(
        required("imported-monorepo-paths"),
        expect(&[("packages/app/src/visibility.mts", 2, VIEW)])
    );
}

#[test]
fn caller_directives_still_suppress_sql_reported_in_the_helper() {
    // File-level, executor-line, and helper-call-line directives in the caller
    // each suppress; only the function with no directive reaches the helper.
    assert_eq!(
        required("imported-suppressed-caller"),
        expect(&[("src/helper.mts", 2, VIEW)])
    );
    assert_eq!(
        findings_of("postgres-no-offset", "imported-suppressed-caller"),
        expect(&[("src/helper.mts", 2, "offset#3")])
    );
    let report = run(REQUIRED, "imported-suppressed-caller", true);
    let suppressed = report["suppressed"].as_array().unwrap();
    // Two in the file-suppressed caller, three in the line-suppressed one.
    assert_eq!(suppressed.len(), 5, "{report}");
    assert!(suppressed
        .iter()
        .all(|entry| entry["file"].as_str().unwrap().starts_with("src/")
            && entry["file"] != "src/helper.mts"));
}

#[test]
fn module_level_string_constants_are_judged_and_computed_forms_stay_unanalyzable() {
    let unknown = [
        86, 92, 100, 106, 112, 118, 124, 129, 133, 137, 141, 146, 151,
    ];
    let mut rows = vec![("src/caller.mts", 74, "orders")];
    rows.extend(
        unknown
            .into_iter()
            .map(|line| ("src/caller.mts", line, UNANALYZABLE)),
    );
    assert_eq!(required("module-const-sql"), expect(&rows));
    let report = run(REQUIRED, "module-const-sql", true);
    let suppressed = report["suppressed"].as_array().unwrap();
    assert!(
        suppressed.iter().any(|entry| {
            entry["file"] == "src/caller.mts"
                && entry["line"].as_u64() == Some(81)
                && entry["reason"].as_str().is_some_and(|reason| {
                    reason.contains("orders") && reason.contains("account_id")
                })
        }),
        "{report}"
    );
}

#[test]
fn an_iife_builder_with_a_conditional_and_several_imported_helpers_is_recovered() {
    // Both reads of the view: the CTE in the caller (inside the IIFE and
    // outside it) and the EXISTS in the helper reached through a paths alias.
    assert_eq!(
        required("imported-peer-shape"),
        expect(&[
            ("src/caller.mts", 15, VIEW),
            ("src/caller.mts", 31, VIEW),
            ("src/services/links/visibility-sql.mts", 2, VIEW),
        ])
    );
}

#[test]
fn a_directly_invoked_function_returning_its_local_builder_hands_over_the_builder() {
    assert_eq!(
        required("imported-iife-builder"),
        expect(&[
            ("src/caller.mts", 9, VIEW),
            ("src/caller.mts", 18, VIEW),
            ("src/visibility-sql.mts", 2, VIEW),
        ])
    );
}

#[test]
fn iifes_outside_the_exact_shape_stay_unanalyzable() {
    assert_eq!(
        required("imported-iife-fail-closed"),
        unanalyzable(&[12, 20, 28, 36, 44, 52, 62, 71, 79, 88, 96, 104])
    );
}

#[test]
fn an_imported_helper_after_a_conditional_fills_every_alternative() {
    assert_eq!(
        required("imported-after-conditional"),
        expect(&[
            ("src/caller.mts", 19, VIEW),
            ("src/caller.mts", 34, UNANALYZABLE),
            ("src/visibility-sql.mts", 2, VIEW),
        ])
    );
}

#[test]
fn non_literal_or_missing_arguments_stay_unanalyzable() {
    assert_eq!(
        required("imported-nonliteral-arg"),
        unanalyzable(&[10, 18, 24, 30, 37])
    );
}

#[test]
fn parameters_used_outside_an_interpolation_stay_unanalyzable() {
    // Concatenation with a module constant and a method call on the parameter
    // stay opaque; a plain local alias of the parameter is still a literal.
    let mut expected = unanalyzable(&[10, 16]);
    expected.extend(expect(&[("src/visibility-sql.mts", 13, VIEW)]));
    expected.sort();
    assert_eq!(required("imported-param-outside-interpolation"), expected);
}

#[test]
fn bounded_helper_control_flow_is_recovered_and_loops_stay_unanalyzable() {
    // `branching` returns on an unknown test (two alternatives) and
    // `throwing` ends the path that throws; the loop is never unrolled.
    let mut expected = unanalyzable(&[15]);
    expected.extend(expect(&[
        ("src/visibility-sql.mts", 3, VIEW),
        ("src/visibility-sql.mts", 18, VIEW),
    ]));
    expected.sort();
    assert_eq!(required("imported-control-flow"), expected);
}

#[test]
fn unresolved_ambiguous_and_unsupported_imports_stay_unanalyzable() {
    assert_eq!(
        required("imported-unresolved"),
        unanalyzable(&[15, 21, 27, 33, 40, 47, 53])
    );
    assert_eq!(required("imported-barrel-ambiguous"), unanalyzable(&[10]));
}

#[test]
fn barrel_cycles_fail_closed() {
    assert_eq!(required("imported-barrel-cycle"), unanalyzable(&[10]));
}

#[test]
fn re_exports_are_followed_up_to_the_documented_cap_and_no_further() {
    assert_eq!(
        required("imported-barrel-depth-cap"),
        expect(&[("src/chain/declare.mts", 2, VIEW)])
    );
    assert_eq!(
        required("imported-barrel-past-depth-cap"),
        unanalyzable(&[9])
    );
}

#[test]
fn other_dml_rules_share_the_recovery_and_name_the_helper_module() {
    assert_eq!(
        findings_of("postgres-no-offset", "imported-helper"),
        expect(&[("src/paging.mts", 3, "offset")])
    );
    assert_eq!(
        findings_of("postgres-explicit-columns", "imported-helper"),
        expect(&[("src/lateral.mts", 3, "accounts")])
    );
    // The helper only adds ORDER BY, so the recovered query has no LIMIT.
    assert_eq!(
        findings_of("postgres-bounded-statements", "imported-helper"),
        expect(&[("src/caller.mts", 7, "table:accounts")])
    );
}

#[test]
fn a_spliced_caller_literal_is_credited_to_the_literal_not_the_placeholder() {
    // `OFFSET` is written only in the caller's quoted string.
    assert_eq!(
        findings_of("postgres-no-offset", "imported-spliced-literal"),
        expect(&[("src/caller.mts", 8, "offset")])
    );
    assert_eq!(
        findings_of("postgres-no-offset", "same-file-spliced-literal"),
        expect(&[("src/caller.mts", 12, "offset")])
    );
    assert_eq!(
        findings_of("postgres-no-offset", "imported-spliced-backtick"),
        expect(&[("src/caller.mts", 8, "offset")])
    );
    assert_eq!(
        findings_of("postgres-no-offset", "same-file-spliced-backtick"),
        expect(&[("src/caller.mts", 12, "offset")])
    );
}

#[test]
fn one_function_appending_one_helper_twice_merges_and_one_directive_suppresses_it() {
    // Both appends are one statement of the same helper token, so one finding.
    assert_eq!(
        required("imported-repeated-helper"),
        expect(&[("src/visibility-sql.mts", 2, VIEW)])
    );
    let report = run(REQUIRED, "imported-repeated-helper", true);
    let suppressed = report["suppressed"].as_array().unwrap();
    // Disabling one append suppresses the whole statement. Accounting stays on
    // that append (line 22), not on the other call or the helper.
    assert_eq!(suppressed.len(), 1, "{report}");
    assert_eq!(suppressed[0]["file"], "src/caller.mts");
    assert_eq!(suppressed[0]["line"], 22);
    assert_eq!(suppressed[0]["directive"]["line"], 21);
}
