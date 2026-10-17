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
fn non_literal_or_missing_arguments_stay_unanalyzable() {
    assert_eq!(
        required("imported-nonliteral-arg"),
        unanalyzable(&[10, 18, 24, 30])
    );
}

#[test]
fn parameters_used_outside_an_interpolation_stay_unanalyzable() {
    assert_eq!(
        required("imported-param-outside-interpolation"),
        unanalyzable(&[10, 16, 22])
    );
}

#[test]
fn helpers_with_control_flow_stay_unanalyzable() {
    assert_eq!(
        required("imported-control-flow"),
        unanalyzable(&[9, 15, 21])
    );
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
