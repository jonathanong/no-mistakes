//! Saved-fixture tests and snapshots for LCOV merging and deterministic serialization (#4800 §A20, §B).

mod common;

use common::{pretty_json, read_fixture_bytes};
use no_mistakes_coverage::{
    merge_reports, parse_lcov_bytes, parse_lcov_str, to_lcov, to_lcov_minimal, BranchKey,
    CoverageReport, ParseOptions,
};

fn parse_fixture_report(name: &str) -> CoverageReport {
    let parsed = parse_lcov_bytes(&read_fixture_bytes(name), &ParseOptions::default());
    assert!(
        parsed.diagnostics.is_empty(),
        "unexpected diagnostics for {name}: {:?}",
        parsed.diagnostics
    );
    parsed.report
}

#[test]
fn merges_da_records_with_differing_optional_checksums() {
    let a = parse_fixture_report("da_optional_checksum/shard_a.lcov");
    let b = parse_fixture_report("da_optional_checksum/shard_b.lcov");
    let merged = merge_reports([&a, &b]);
    assert_eq!(merged.files["src/checksum.ts"].lines[&1], 5);
    assert_eq!(merged.files["src/checksum.ts"].lines[&2], 4);
    insta::assert_snapshot!("merge_da_optional_checksum", pretty_json(&merged));
}

#[test]
fn merges_sections_with_different_tn_names() {
    let unit = parse_fixture_report("merge_different_tn/shard_unit.lcov");
    let e2e = parse_fixture_report("merge_different_tn/shard_e2e.lcov");
    let merged = merge_reports([&unit, &e2e]);
    assert_eq!(merged.len(), 1);
    let cov = &merged.files["src/a.ts"];
    assert_eq!(cov.lines[&1], 5);
    assert_eq!(cov.functions["foo"].hits, 5);
    assert_eq!(cov.branches[&BranchKey::new(3, "0", "0")], 5);
    insta::assert_snapshot!("merge_different_tn", pretty_json(&merged));
}

#[test]
fn merges_disjoint_shards_and_saturates_hit_overflow() {
    let a = parse_fixture_report("merge_shards/shard_a.lcov");
    let b = parse_fixture_report("merge_shards/shard_b.lcov");
    let merged_ab = merge_reports([&a, &b]);
    assert_eq!(merged_ab.len(), 3);
    insta::assert_snapshot!("merge_shards_ab", pretty_json(&merged_ab));

    let overflow = parse_fixture_report("merge_shards/shard_overflow.lcov");
    let saturated = merge_reports([&merged_ab, &overflow]);
    let shared = &saturated.files["src/shared.ts"];
    assert_eq!(shared.lines[&1], u64::MAX);
    assert_eq!(shared.functions["fnA"].hits, u64::MAX);
    assert_eq!(shared.branches[&BranchKey::new(1, "0", "0")], u64::MAX);
    insta::assert_snapshot!("merge_shards_saturated", pretty_json(&saturated));
}

#[test]
fn keeps_first_seen_fn_line_and_backfills_missing_fn_def() {
    let a = parse_fixture_report("merge_fn_line_conflict/shard_a.lcov");
    let b = parse_fixture_report("merge_fn_line_conflict/shard_b.lcov");
    let merged = merge_reports([&a, &b]);
    let fns = &merged.files["src/conflict.ts"].functions;
    assert_eq!(fns["foo"].line, Some(1));
    assert_eq!(fns["foo"].end_line, Some(5));
    assert_eq!(fns["foo"].hits, 5);
    assert_eq!(fns["lateDef"].line, Some(20));
    assert_eq!(fns["lateDef"].end_line, Some(30));
    assert_eq!(fns["lateDef"].hits, 5);
    insta::assert_snapshot!("merge_fn_line_conflict", pretty_json(&merged));
}

#[test]
fn merges_empty_and_single_report_inputs() {
    let empty = merge_reports(std::iter::empty::<&CoverageReport>());
    assert!(empty.is_empty());
    assert_eq!(to_lcov(&empty), "");
    assert_eq!(to_lcov_minimal(&empty), "");

    let single = parse_fixture_report("empty_sf_section.lcov");
    let cloned = merge_reports([&single]);
    assert_eq!(cloned, single);
}

#[test]
fn serializes_deterministically_in_full_and_minimal_forms() {
    let report = parse_fixture_report("deterministic_serialization.lcov");
    let full = to_lcov(&report);
    let minimal = to_lcov_minimal(&report);
    insta::assert_snapshot!("serialize_full", full);
    insta::assert_snapshot!("serialize_minimal", minimal);

    let reparsed = parse_lcov_str(&full, &ParseOptions::default());
    assert!(reparsed.diagnostics.is_empty());
    assert_eq!(reparsed.report, report);
}

#[test]
fn round_trips_coverage_report_and_branch_keys_through_json() {
    let report = parse_fixture_report("brda_non_numeric_block.lcov");
    let json = serde_json::to_string(&report).expect("serialize report");
    let round_tripped: CoverageReport = serde_json::from_str(&json).expect("deserialize report");
    assert_eq!(round_tripped, report);

    assert!(BranchKey::from_lcov_key("10,0,0").is_some());
    assert!(BranchKey::from_lcov_key("10,0").is_none());
    assert!(BranchKey::from_lcov_key("abc,0,0").is_none());
    assert!(BranchKey::from_lcov_key("10,,0").is_none());
    assert!(BranchKey::from_lcov_key("10,0,").is_none());
    assert!(BranchKey::from_lcov_key("10,0,0,extra").is_none());

    let bad_key: Result<BranchKey, _> = serde_json::from_str("\"invalid\"");
    assert!(bad_key.is_err());

    let a = BranchKey::new(1, "02", "0");
    let b = BranchKey::new(1, "2", "0");
    assert!(a < b);
}
