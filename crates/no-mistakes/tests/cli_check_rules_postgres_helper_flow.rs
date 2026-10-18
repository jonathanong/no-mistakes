//! Executor SQL recovered through builder `.text` and helper control flow.
//! Every scenario is a saved fixture under `test-cases/rules/<rule>/variants/`.

use std::path::PathBuf;
use std::process::Command;

const REQUIRED: &str = "postgres-required-predicates";
const VIEW: &str = "view_active_plans";
const UNANALYZABLE: &str = "unanalyzable";

/// `(file, line, target)` of every finding, sorted.
type Findings = Vec<(String, u64, String)>;

fn required(name: &str) -> Findings {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules")
        .join(REQUIRED)
        .join("variants")
        .join(name);
    let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["check", "--format", "json", "--root"])
        .arg(&root)
        .arg("--config")
        .arg(root.join(".no-mistakes.yml"))
        .output()
        .unwrap();
    let report: serde_json::Value = serde_json::from_slice(&output.stdout)
        .unwrap_or_else(|error| panic!("{error}: {}", String::from_utf8_lossy(&output.stderr)));
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

fn expect(rows: &[(&str, u64, &str)]) -> Findings {
    let mut rows: Findings = rows
        .iter()
        .map(|(file, line, target)| (file.to_string(), *line, target.to_string()))
        .collect();
    rows.sort();
    rows
}

#[test]
fn builder_text_executor_arguments_are_recovered_and_fail_closed() {
    // Lines 6 and 13 are the builders' own SQL; every later case must stay
    // unanalyzable (unknown builder, extra interpolation, foreign `.values`,
    // plain object, partially recovered builder).
    assert_eq!(
        required("builder-text"),
        expect(&[
            ("src/caller.mts", 6, VIEW),
            ("src/caller.mts", 13, VIEW),
            ("src/caller.mts", 22, UNANALYZABLE),
            ("src/caller.mts", 28, UNANALYZABLE),
            ("src/caller.mts", 35, UNANALYZABLE),
            ("src/caller.mts", 41, UNANALYZABLE),
            ("src/caller.mts", 48, UNANALYZABLE),
            // Missing, extra or wrapped `.values`, a repeated `.text`, a
            // computed member and a call inside a loop.
            ("src/caller.mts", 55, UNANALYZABLE),
            ("src/caller.mts", 60, UNANALYZABLE),
            ("src/caller.mts", 65, UNANALYZABLE),
            ("src/caller.mts", 70, UNANALYZABLE),
            ("src/caller.mts", 75, UNANALYZABLE),
            ("src/caller.mts", 81, UNANALYZABLE),
        ])
    );
}

#[test]
fn helper_control_flow_forks_on_unknown_tests_and_decides_known_ones() {
    // Lines 10 and 18 are the callers' own SQL; `list-query.mts:6` and the
    // same-file window arm are the early-return alternatives; only the strict
    // arm of `decided.mts` can run for a literal `true`. Line 47 uses a
    // helper's flag as a value, which does not return the builder it changed.
    assert_eq!(
        required("helper-control-flow"),
        expect(&[
            ("src/caller.mts", 10, VIEW),
            ("src/caller.mts", 18, VIEW),
            ("src/caller.mts", 47, UNANALYZABLE),
            ("src/decided.mts", 6, VIEW),
            ("src/list-query.mts", 6, VIEW),
        ])
    );
}

#[test]
fn nested_helper_composition_is_bounded_by_the_depth_limit() {
    // The guarded view is read three helpers down; eight chained helpers
    // recover and a ninth does not.
    assert_eq!(
        required("helper-composition"),
        expect(&[
            ("src/caller.mts", 42, UNANALYZABLE),
            ("src/chain-ok.mts", 35, VIEW),
            ("src/clauses.mts", 22, VIEW),
        ])
    );
}

#[test]
fn helper_composition_boundaries_stay_unanalyzable() {
    // cycle, loop over a parameter, unbounded loop, more than sixteen
    // alternatives, builder in an untyped parameter, undeclared callee,
    // call through a parameter, identifier in raw text, rebound tags and an
    // unresolved callee handed the builder.
    let lines = [21, 25, 29, 33, 39, 43, 47, 51, 55, 59, 65];
    let rows: Vec<_> = lines
        .iter()
        .map(|line| ("src/caller.mts", *line, UNANALYZABLE))
        .collect();
    assert_eq!(required("helper-composition-fail"), expect(&rows));
}

#[test]
fn object_literal_properties_splice_as_identifiers_only_when_literal() {
    assert_eq!(
        required("helper-object-identifiers"),
        expect(&[
            ("src/caller.mts", 13, VIEW),
            ("src/caller.mts", 31, UNANALYZABLE),
            ("src/caller.mts", 39, UNANALYZABLE),
            ("src/caller.mts", 44, UNANALYZABLE),
        ])
    );
}

#[test]
fn fluent_statement_chains_append_every_link_to_a_tracked_builder() {
    assert_eq!(
        required("chained-append-statement"),
        expect(&[
            ("src/caller.mts", 10, VIEW),
            ("src/caller.mts", 17, VIEW),
            ("src/caller.mts", 25, VIEW),
            ("src/caller.mts", 33, UNANALYZABLE),
        ])
    );
}

#[test]
fn workspace_package_names_resolve_through_the_workspace_map() {
    assert_eq!(
        required("imported-workspace-package"),
        expect(&[
            ("packages/app/src/caller.mts", 25, UNANALYZABLE),
            ("packages/visibility/owner.mts", 2, VIEW),
        ])
    );
}

#[test]
fn loops_unroll_only_over_short_literal_arrays() {
    assert_eq!(
        required("helper-literal-loops"),
        expect(&[
            ("src/caller.mts", 14, UNANALYZABLE),
            ("src/caller.mts", 19, UNANALYZABLE),
            ("src/union.mts", 7, VIEW),
        ])
    );
}

#[test]
fn an_undefined_argument_takes_the_parameter_default() {
    // Only the `requireLive` arm (helper line 6) runs; line 8 would appear if
    // `undefined` were treated as a falsy value.
    assert_eq!(
        required("helper-default-parameter"),
        expect(&[("src/gated.mts", 6, VIEW)])
    );
}

#[test]
fn helper_call_arguments_are_classified_before_a_hole_is_recorded() {
    // The four reasonable calls recover; every other shape leaves the call
    // unanalyzable. The helper's own line is the only view finding.
    let lines = [44, 50, 56, 62, 68, 74, 80, 87, 94, 100, 106, 112, 118];
    let mut rows: Vec<_> = lines
        .iter()
        .map(|line| ("src/caller.mts", *line, UNANALYZABLE))
        .collect();
    rows.push(("src/gate.mts", 4, VIEW));
    assert_eq!(required("helper-argument-shapes"), expect(&rows));
}

#[test]
fn hole_alternatives_multiply_up_to_the_cap() {
    // Four independent two-way holes are sixteen statements and recover;
    // a fifth is thirty-two and does not.
    assert_eq!(
        required("helper-alternatives-cap"),
        expect(&[
            ("src/caller.mts", 27, UNANALYZABLE),
            ("src/maybe.mts", 3, VIEW),
        ])
    );
}
