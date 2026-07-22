//! Direct unit tests for the artifact-dataflow subsystem's pure logic that
//! the golden-JSON fixtures in `tests.rs` can't reach cheaply: brace-pattern
//! expansion boundaries, matrix-axis validation edge cases, and a few
//! resolution-helper fallbacks. Mirrors the vendored TS engine's own
//! `artifact-coverage.test.mts` / `artifact-resolution-helpers.test.mts`.

use super::super::artifact_pattern_match::matches_artifact_pattern;
use super::super::artifact_resolution_helpers::{
    diagnostic_key, occurrence_reaches, symbolic_pattern_match,
};
use super::super::artifact_resolution_types::ArtifactRunContext;
use super::super::artifact_values::{
    artifact_value, parse_artifact_declaration, static_matrix_instance_count,
};
use super::super::value_primitives::{to_json, yaml_number_to_json, OrderedJson};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

fn matrix(entries: &[(&str, OrderedJson)]) -> OrderedJson {
    OrderedJson::Object(
        entries
            .iter()
            .map(|(key, value)| (key.to_string(), value.clone()))
            .collect(),
    )
}

fn string_array(values: &[&str]) -> OrderedJson {
    OrderedJson::Array(
        values
            .iter()
            .map(|v| OrderedJson::String(v.to_string()))
            .collect(),
    )
}

// ── artifact_pattern_match: brace expansion ─────────────────────────────

#[test]
fn matches_a_plain_glob_with_no_braces() {
    assert!(matches_artifact_pattern("build-linux", "build-*"));
    assert!(!matches_artifact_pattern("docs", "build-*"));
}

#[test]
fn rejects_a_pattern_over_the_length_limit() {
    let pattern = "x".repeat(1025);
    assert!(!matches_artifact_pattern("x", &pattern));
}

#[test]
fn expands_a_comma_list_and_matches_any_alternative() {
    assert!(matches_artifact_pattern(
        "build-macos",
        "build-{linux,macos,windows}"
    ));
    assert!(!matches_artifact_pattern(
        "build-freebsd",
        "build-{linux,macos,windows}"
    ));
}

#[test]
fn expands_nested_braces() {
    assert!(matches_artifact_pattern("report-b", "report-{a,{b,c}}"));
    assert!(matches_artifact_pattern("report-c", "report-{a,{b,c}}"));
}

#[test]
fn expands_a_numeric_range_including_a_reverse_range_and_step() {
    assert!(matches_artifact_pattern("build-3", "build-{1..5}"));
    assert!(!matches_artifact_pattern("build-6", "build-{1..5}"));
    assert!(matches_artifact_pattern("build-4", "build-{5..1}"));
    assert!(matches_artifact_pattern("build-1", "build-{1..5..2}"));
    assert!(!matches_artifact_pattern("build-2", "build-{1..5..2}"));
}

#[test]
fn expands_an_alpha_range() {
    assert!(matches_artifact_pattern("shard-c", "shard-{a..d}"));
    assert!(!matches_artifact_pattern("shard-e", "shard-{a..d}"));
}

#[test]
fn leaves_a_single_entry_or_unbalanced_brace_literal() {
    // `a{b}c` has no comma/range inside — bash keeps it literal, so this is
    // a plain (non-matching, since the name has no literal braces) glob.
    assert!(!matches_artifact_pattern("abc", "a{b}c"));
    assert!(matches_artifact_pattern("a{b}c", "a{b}c"));
    // Unbalanced `{` never closes — also literal.
    assert!(matches_artifact_pattern("build-{linux", "build-{linux"));
}

#[test]
fn treats_an_over_limit_brace_pattern_as_a_conservative_non_match() {
    let adjacent = format!("report-{}", "{a,b}".repeat(9));
    assert!(!matches_artifact_pattern("report-aaaaaaaaa", &adjacent));
    let nested = format!("report-{}", "{a,{b,c}}".repeat(6));
    assert!(!matches_artifact_pattern("report-aaaaaa", &nested));
    assert!(!matches_artifact_pattern("report-1", "report-{1..300}"));
}

#[test]
fn loads_deep_sequential_braces_as_a_conservative_non_matching_pattern_without_hanging() {
    let pattern = "{a,b}".repeat(10_000);
    assert!(!matches_artifact_pattern("anything", &pattern));
}

// ── artifact_resolution_helpers ──────────────────────────────────────────

#[test]
fn diagnostic_key_of_none_is_empty() {
    assert_eq!(diagnostic_key(None), "");
}

#[test]
fn occurrence_reaches_caches_the_full_reachable_set() {
    let context = ArtifactRunContext {
        occurrences: Vec::new(),
        adjacency: HashMap::from([
            ("a".to_string(), HashSet::from(["b".to_string()])),
            ("b".to_string(), HashSet::from(["c".to_string()])),
            ("c".to_string(), HashSet::new()),
        ]),
        reachability_cache: RefCell::new(HashMap::new()),
        complete: true,
    };
    assert!(occurrence_reaches(&context, "a", "c"));
    assert!(context.reachability_cache.borrow().contains_key("a"));
    assert!(occurrence_reaches(&context, "a", "b"));
    assert!(!occurrence_reaches(&context, "absent", "missing"));
}

#[test]
fn symbolic_pattern_match_requires_at_least_one_expression() {
    assert!(!symbolic_pattern_match("literal", "*"));
}

#[test]
fn symbolic_pattern_match_rejects_glob_metacharacters_in_literal_segments() {
    assert!(!symbolic_pattern_match("a[${{ inputs.x }}]", "a[*]"));
    assert!(!symbolic_pattern_match("${{ inputs.name }}-[x]", "*-x"));
}

#[test]
fn symbolic_pattern_match_requires_a_leading_wildcard_after_each_expression() {
    assert!(!symbolic_pattern_match(
        "report-${{ inputs.name }}",
        "report-name"
    ));
}

// ── artifact_values: matrix axis validation ──────────────────────────────

#[test]
fn parse_artifact_declaration_ignores_a_non_artifact_action() {
    assert!(parse_artifact_declaration("example/action@v1", None, None).is_none());
}

#[test]
fn simple_matrix_axes_reject_include_exclude_and_invalid_shapes() {
    // `include`/`exclude` make the real combination set impossible to
    // enumerate this way.
    let with_include = matrix(&[
        ("os", string_array(&["linux"])),
        ("include", OrderedJson::Array(Vec::new())),
    ]);
    assert_eq!(static_matrix_instance_count(Some(&with_include)), None);

    // An empty axis has no combinations.
    let empty_axis = matrix(&[("os", OrderedJson::Array(Vec::new()))]);
    assert_eq!(static_matrix_instance_count(Some(&empty_axis)), None);

    // A non-array axis value isn't a matrix axis.
    let non_array = matrix(&[("os", OrderedJson::String("linux".to_string()))]);
    assert_eq!(static_matrix_instance_count(Some(&non_array)), None);

    // An axis item that isn't a string/number/bool is rejected.
    let bad_item = matrix(&[("os", OrderedJson::Array(vec![OrderedJson::Null]))]);
    assert_eq!(static_matrix_instance_count(Some(&bad_item)), None);

    // Over the 256-combination cap.
    let oversized = matrix(&[(
        "os",
        OrderedJson::Array((0..257).map(|n| OrderedJson::Number(n.into())).collect()),
    )]);
    assert_eq!(static_matrix_instance_count(Some(&oversized)), None);

    // No matrix at all: exactly one (unmatrixed) instance.
    assert_eq!(static_matrix_instance_count(None), Some(1));

    // A valid two-axis matrix multiplies combination counts.
    let valid = matrix(&[
        ("os", string_array(&["linux", "macos"])),
        (
            "node",
            OrderedJson::Array(vec![
                OrderedJson::Number(22.into()),
                OrderedJson::Number(24.into()),
            ]),
        ),
    ]);
    assert_eq!(static_matrix_instance_count(Some(&valid)), Some(4));
}

#[test]
fn artifact_value_treats_a_reference_to_a_missing_axis_as_dynamic() {
    let axes = matrix(&[("os", string_array(&["linux"]))]);
    assert!(matches!(
        artifact_value("${{ matrix.missing }}", Some(&axes)),
        super::super::artifact_types::ArtifactValue::Dynamic { .. }
    ));
    // A reference to a real axis mixed with a non-matrix expression can
    // never be fully resolved either.
    assert!(matches!(
        artifact_value("${{ matrix.os }}-${{ inputs.name }}", Some(&axes)),
        super::super::artifact_types::ArtifactValue::Dynamic { .. }
    ));
}

// ── value_primitives: to_json/yaml_number_to_json fallbacks ─────────────
//
// These branches are reachability-guaranteed-safe fallbacks for YAML shapes
// that never appear in a real GitHub Actions workflow (a `!!tag`ged value,
// a non-string mapping key, a NaN/Infinity float) — hand-constructed here
// since no realistic fixture reaches them.

#[test]
fn to_json_unwraps_a_tagged_value() {
    let tagged = serde_yaml::Value::Tagged(Box::new(serde_yaml::value::TaggedValue {
        tag: serde_yaml::value::Tag::new("custom"),
        value: serde_yaml::Value::String("payload".to_string()),
    }));
    assert_eq!(to_json(&tagged), OrderedJson::String("payload".to_string()));
}

#[test]
fn to_json_stringifies_a_non_string_mapping_key() {
    let mut mapping = serde_yaml::Mapping::new();
    mapping.insert(
        serde_yaml::Value::Bool(true),
        serde_yaml::Value::String("v".to_string()),
    );
    let value = to_json(&serde_yaml::Value::Mapping(mapping));
    let OrderedJson::Object(entries) = value else {
        panic!("expected an object");
    };
    assert_eq!(
        entries,
        vec![("true".to_string(), OrderedJson::String("v".to_string()))]
    );
}

#[test]
fn yaml_number_to_json_rejects_a_non_finite_float() {
    let nan = serde_yaml::Number::from(f64::NAN);
    assert_eq!(yaml_number_to_json(&nan), None);
}
