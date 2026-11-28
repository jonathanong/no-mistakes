//! Saved-fixture tests and snapshots for LCOV parsing edge cases (#4800 Checklist §A).

mod common;

use common::{pretty_json, read_fixture_bytes};
use no_mistakes_coverage::{parse_lcov_bytes, BranchKey, ParseDiagnosticKind, ParseOptions};

fn parse_fixture(name: &str) -> no_mistakes_coverage::ParsedLcov {
    let bytes = read_fixture_bytes(name);
    parse_lcov_bytes(&bytes, &ParseOptions::default())
}

#[test]
fn parses_standard_records() {
    let parsed = parse_fixture("standard_records.lcov");
    assert!(parsed.diagnostics.is_empty());
    assert_eq!(parsed.report.len(), 2);
    let foo = &parsed.report.files["src/foo.mts"];
    assert_eq!(foo.lines_found(), 4);
    assert_eq!(foo.lines_hit(), 3);
    assert_eq!(foo.functions_found(), 1);
    assert_eq!(foo.functions_hit(), 1);
    assert_eq!(foo.branches_found(), 2);
    assert_eq!(foo.branches_hit(), 1);
    insta::assert_snapshot!("standard_records", pretty_json(&parsed));
}

#[test]
fn parses_empty_input() {
    let parsed = parse_fixture("empty_input.lcov");
    assert!(parsed.report.is_empty());
    assert!(parsed.diagnostics.is_empty());
    insta::assert_snapshot!("empty_input", pretty_json(&parsed));
}

#[test]
fn parses_crlf_and_missing_trailing_newline() {
    let parsed = parse_fixture("crlf_no_trailing_newline.lcov");
    assert!(parsed.diagnostics.is_empty());
    assert_eq!(parsed.report.len(), 2);
    insta::assert_snapshot!("crlf_no_trailing_newline", pretty_json(&parsed));
}

#[test]
fn parses_brda_dash_as_zero_and_merges_hits() {
    let parsed = parse_fixture("brda_dash_untaken.lcov");
    assert!(parsed.diagnostics.is_empty());
    let cov = &parsed.report.files["src/branches.ts"];
    assert_eq!(cov.branches[&BranchKey::new(10, "0", "0")], 2);
    assert_eq!(cov.branches[&BranchKey::new(10, "0", "1")], 5);
    assert_eq!(cov.branches[&BranchKey::new(12, "0", "0")], 0);
    insta::assert_snapshot!("brda_dash_untaken", pretty_json(&parsed));
}

#[test]
fn parses_trailing_whitespace_spaces_and_tabs() {
    let parsed = parse_fixture("trailing_whitespace.lcov");
    assert!(parsed.diagnostics.is_empty());
    assert!(parsed.report.files.contains_key("src/ws.ts"));
    insta::assert_snapshot!("trailing_whitespace", pretty_json(&parsed));
}

#[test]
fn skips_blank_and_unknown_records() {
    let parsed = parse_fixture("blank_and_unknown_lines.lcov");
    assert!(parsed.diagnostics.is_empty());
    assert_eq!(parsed.report.files["src/a.ts"].lines[&1], 1);
    insta::assert_snapshot!("blank_and_unknown_lines", pretty_json(&parsed));
}

#[test]
fn commits_open_record_when_missing_end_of_record() {
    let parsed = parse_fixture("missing_end_of_record.lcov");
    assert!(parsed.diagnostics.is_empty());
    assert_eq!(parsed.report.files["src/no_eor.ts"].lines[&1], 2);
    insta::assert_snapshot!("missing_end_of_record", pretty_json(&parsed));
}

#[test]
fn commits_previous_record_on_consecutive_sf_without_eor() {
    let parsed = parse_fixture("consecutive_sf_without_eor.lcov");
    assert!(parsed.diagnostics.is_empty());
    assert_eq!(parsed.report.len(), 2);
    insta::assert_snapshot!("consecutive_sf_without_eor", pretty_json(&parsed));
}

#[test]
fn records_orphan_diagnostics_outside_sf_blocks() {
    let parsed = parse_fixture("orphan_records.lcov");
    assert_eq!(parsed.report.len(), 1);
    assert!(!parsed.report.files["src/valid.ts"].lines.contains_key(&1));
    assert_eq!(parsed.report.files["src/valid.ts"].lines[&2], 1);
    assert_eq!(parsed.diagnostics.len(), 5);
    assert!(parsed
        .diagnostics
        .iter()
        .all(|d| d.kind == ParseDiagnosticKind::OrphanRecord && d.record_path.is_none()));
    insta::assert_snapshot!("orphan_records", pretty_json(&parsed));
}

#[test]
fn merges_duplicate_sf_sections_within_one_file() {
    let parsed = parse_fixture("duplicate_sf_same_file.lcov");
    assert!(parsed.diagnostics.is_empty());
    let cov = &parsed.report.files["web/components/Foo.tsx"];
    assert_eq!(cov.lines[&1], 5);
    assert_eq!(cov.lines[&2], 1);
    assert_eq!(cov.functions["render"].hits, 5);
    assert_eq!(cov.branches[&BranchKey::new(3, "0", "0")], 5);
    insta::assert_snapshot!("duplicate_sf_same_file", pretty_json(&parsed));
}

#[test]
fn sums_duplicate_da_lines_within_one_section() {
    let parsed = parse_fixture("duplicate_da_same_section.lcov");
    assert!(parsed.diagnostics.is_empty());
    let cov = &parsed.report.files["src/dup_da.ts"];
    assert_eq!(cov.lines[&1], 3);
    assert_eq!(cov.lines[&2], 5);
    insta::assert_snapshot!("duplicate_da_same_section", pretty_json(&parsed));
}

#[test]
fn handles_fn_ordering_duplicates_and_unhit_functions() {
    let parsed = parse_fixture("fn_ordering_and_unhit.lcov");
    assert!(parsed.diagnostics.is_empty());
    let fns = &parsed.report.files["src/fn_edges.ts"].functions;
    assert_eq!(fns["unhitOnly"].line, Some(1));
    assert_eq!(fns["unhitOnly"].hits, 0);
    assert_eq!(fns["dupHits"].hits, 6);
    assert_eq!(fns["fnAfterFnda"].line, Some(20));
    assert_eq!(fns["fnAfterFnda"].hits, 5);
    assert_eq!(fns["orphanFnda"].line, None);
    assert_eq!(fns["orphanFnda"].hits, 3);
    insta::assert_snapshot!("fn_ordering_and_unhit", pretty_json(&parsed));
}

#[test]
fn discards_vitest_empty_report_placeholders_without_clobbering_genuine_records() {
    let parsed = parse_fixture("vitest_empty_report_placeholder.lcov");
    assert!(parsed.diagnostics.is_empty());
    assert!(!parsed.report.files.contains_key("src/unrun.ts"));
    assert!(!parsed.report.files.contains_key("src/fnda_placeholder.ts"));
    assert_eq!(parsed.report.files["src/run_after.ts"].lines[&1], 2);
    assert_eq!(parsed.report.files["src/run_before.ts"].lines[&1], 4);
    insta::assert_snapshot!("vitest_empty_report_placeholder", pretty_json(&parsed));
}

#[test]
fn preserves_empty_sf_sections() {
    let parsed = parse_fixture("empty_sf_section.lcov");
    assert!(parsed.diagnostics.is_empty());
    assert!(parsed.report.files.contains_key("src/empty.ts"));
    assert_eq!(parsed.report.files["src/empty.ts"].lines_found(), 0);
    insta::assert_snapshot!("empty_sf_section", pretty_json(&parsed));
}

#[test]
fn parses_lcov2_fn_fnl_and_fna_records() {
    let parsed = parse_fixture("lcov2_fn_fnl_fna.lcov");
    assert!(parsed.diagnostics.is_empty());
    let fns = &parsed.report.files["src/lcov2.ts"].functions;
    assert_eq!(fns["threeFieldFn"].line, Some(1));
    assert_eq!(fns["threeFieldFn"].end_line, Some(5));
    assert_eq!(fns["threeFieldFn"].hits, 2);
    assert_eq!(fns["indexedFn"].line, Some(10));
    assert_eq!(fns["indexedFn"].end_line, Some(15));
    assert_eq!(fns["indexedFn"].hits, 4);
    assert_eq!(fns["indexedNoEndFn"].line, Some(18));
    assert_eq!(fns["indexedNoEndFn"].end_line, None);
    assert_eq!(fns["indexedNoEndFn"].hits, 0);
    assert_eq!(fns["legacyFnlArrow"].line, Some(25));
    assert_eq!(fns["legacyFnlArrow"].hits, 6);
    assert_eq!(fns["aliasFn"].hits, 3);
    assert_eq!(fns["genericFn(a, b)"].line, Some(50));
    insta::assert_snapshot!("lcov2_fn_fnl_fna", pretty_json(&parsed));
}

#[test]
fn parses_brda_with_non_numeric_block_and_branch_ids() {
    let parsed = parse_fixture("brda_non_numeric_block.lcov");
    assert!(parsed.diagnostics.is_empty());
    let keys: Vec<String> = parsed.report.files["src/non_numeric_brda.ts"]
        .branches
        .keys()
        .map(BranchKey::as_lcov_key)
        .collect();
    assert_eq!(
        keys,
        vec![
            "10,2,0",
            "10,10,0",
            "99,blockA,2",
            "99,blockA,10",
            "99,blockA,branchX",
        ]
    );
    insta::assert_snapshot!("brda_non_numeric_block", pretty_json(&parsed));
}

#[test]
fn parses_brda_with_five_or_more_fields() {
    let parsed = parse_fixture("brda_five_fields.lcov");
    assert!(parsed.diagnostics.is_empty());
    let cov = &parsed.report.files["src/five_field_brda.ts"];
    assert_eq!(cov.branches[&BranchKey::new(10, "0", "0")], 3);
    assert_eq!(cov.branches[&BranchKey::new(10, "0", "1")], 0);
    insta::assert_snapshot!("brda_five_fields", pretty_json(&parsed));
}

#[test]
fn scopes_malformed_record_errors_and_preserves_valid_lines() {
    let parsed = parse_fixture("malformed_records.lcov");
    assert_eq!(parsed.report.files["src/malformed.ts"].lines[&1], 7);
    assert_eq!(parsed.report.files["src/after_malformed.ts"].lines[&2], 3);
    assert_eq!(parsed.diagnostics[0].kind, ParseDiagnosticKind::InvalidPath);
    assert!(parsed
        .diagnostics
        .iter()
        .skip(1)
        .all(|d| d.kind == ParseDiagnosticKind::MalformedLine
            && d.record_path.as_deref() == Some("src/malformed.ts")));
    insta::assert_snapshot!("malformed_records", pretty_json(&parsed));
}

#[test]
fn decodes_invalid_utf8_lossily_with_scoped_diagnostic() {
    let parsed = parse_fixture("invalid_utf8.lcov");
    assert_eq!(parsed.diagnostics.len(), 1);
    assert_eq!(parsed.diagnostics[0].kind, ParseDiagnosticKind::InvalidUtf8);
    assert_eq!(parsed.diagnostics[0].line, 4);
    assert_eq!(parsed.report.files["src/before.ts"].lines[&1], 1);
    assert_eq!(parsed.report.files["src/after.ts"].lines[&4], 5);
    assert_eq!(parsed.report.len(), 3);
    insta::assert_snapshot!("invalid_utf8", pretty_json(&parsed));
}
