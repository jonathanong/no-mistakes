use super::{document::Document, workflow, Reference};
use serde_yaml::Value;
use std::sync::Arc;

fn references(fixture: &str) -> Vec<Reference> {
    let source = match fixture {
        "scanner.yml" => include_str!("../../../fixtures/ci-secret-facts/scanner.yml"),
        "malformed-expression.yml" => {
            include_str!("../../../fixtures/ci-secret-facts/malformed-expression.yml")
        }
        "empty-selector.yml" => {
            include_str!("../../../fixtures/ci-secret-facts/empty-selector.yml")
        }
        "unicode-prefix.yml" => {
            include_str!("../../../fixtures/ci-secret-facts/unicode-prefix.yml")
        }
        "escaped-selector.yml" => {
            include_str!("../../../fixtures/ci-secret-facts/escaped-selector.yml")
        }
        _ => unreachable!("known saved fixture"),
    };
    let value: Value = serde_yaml::from_str(source).unwrap();
    let facts = workflow::collect(&value, source, false);
    facts
        .references
        .into_iter()
        .chain(facts.jobs.into_values().flat_map(|scope| scope.references))
        .collect()
}

fn sorted(references: &[Reference]) -> Vec<(usize, Option<&str>)> {
    let mut actual: Vec<_> = references
        .iter()
        .map(|reference| (reference.line, reference.name.as_deref()))
        .collect();
    actual.sort();
    actual
}

#[test]
fn scans_expression_boundaries_and_yaml_value_shapes_at_authored_lines() {
    let references = references("scanner.yml");
    let actual = sorted(&references);
    assert_eq!(
        actual,
        [
            (3, Some("TOP")),
            (6, Some("REAL")),
            (9, Some("DOT")),
            (10, Some("BRACKET")),
            (11, None),
            (12, None),
            (14, Some("SEQ_ONE")),
            (15, Some("SEQ_TWO")),
            (16, Some("TAGGED")),
            (17, Some("ANCHOR")),
            (17, Some("ANCHOR")),
            (21, Some("IF_BARE")),
            (25, Some("REPEAT")),
            (26, Some("REPEAT")),
        ]
    );
}

#[test]
fn unclosed_expression_is_dynamic_at_its_opener() {
    assert_eq!(sorted(&references("malformed-expression.yml")), [(5, None)]);
}

#[test]
fn source_marker_parse_failure_is_explicit() {
    let invalid = include_str!("../../../fixtures/ci-secret-facts/invalid-yaml.yml");
    assert!(Document::parse(invalid).is_none());
}

#[test]
fn empty_quoted_bracket_selector_stays_dynamic() {
    assert_eq!(sorted(&references("empty-selector.yml")), [(3, None)]);
}

#[test]
fn unicode_before_multiline_reference_preserves_authored_line() {
    assert_eq!(
        sorted(&references("unicode-prefix.yml")),
        [(7, Some("UNICODE"))]
    );
}

#[test]
fn escaped_selector_without_authored_match_is_dynamic_at_scalar_line() {
    assert_eq!(sorted(&references("escaped-selector.yml")), [(3, None)]);
}

#[test]
fn malformed_source_and_non_mapping_roots_fail_closed() {
    let valid = include_str!("../../../fixtures/ci-secret-facts/scanner.yml");
    let value: Value = serde_yaml::from_str(valid).unwrap();
    let invalid = include_str!("../../../fixtures/ci-secret-facts/invalid-yaml.yml");
    let failed_markers = workflow::collect(&value, invalid, false);
    assert!(failed_markers.references.is_empty());
    assert!(failed_markers.incomplete[0].1.contains("source locations"));

    let root = include_str!("../../../fixtures/ci-secret-facts/null-root.yml");
    let value: Value = serde_yaml::from_str(root).unwrap();
    let facts = workflow::collect(&value, root, false);
    assert!(facts.incomplete[0].1.contains("root must be a mapping"));
    let document = Document::parse(root).unwrap();
    assert!(document.mapping_value(0, "jobs").is_none());
}

#[test]
fn skips_non_string_workflow_and_job_keys() {
    let source = include_str!("../../../fixtures/ci-secret-facts/non-string-keys.yml");
    let value: Value = serde_yaml::from_str(source).unwrap();
    let facts = workflow::collect(&value, source, false);
    assert!(facts.references.is_empty());
    assert!(facts.jobs.is_empty());
    assert!(facts
        .incomplete
        .iter()
        .any(|(_, reason)| reason.contains("workflow keys")));
    assert!(facts
        .incomplete
        .iter()
        .any(|(_, reason)| reason.contains("job keys")));
}

#[test]
fn marker_shape_mismatches_fail_softly_without_losing_dynamic_facts() {
    let value_source = include_str!("../../../fixtures/ci-secret-facts/value-sequence.yml");
    let value: Value = serde_yaml::from_str(value_source).unwrap();
    let marker_source = include_str!("../../../fixtures/ci-secret-facts/marker-map.yml");
    let facts = workflow::collect(&value, marker_source, false);
    let references = facts.jobs["build"]
        .references
        .iter()
        .map(|reference| (reference.name.as_deref(), reference.line))
        .collect::<Vec<_>>();
    assert!(references.contains(&(None, 1)));
    assert!(references.contains(&(None, 5)));
}

#[test]
fn inventory_locations_fail_closed_for_invalid_and_unsupported_shapes() {
    use super::InventoryLocations;

    let invalid = include_str!("../../../fixtures/ci-secret-facts/invalid-yaml.yml");
    assert_eq!(InventoryLocations::parse(invalid).line("", "ALPHA"), None);

    let unsupported = include_str!("../../../fixtures/ci-secret-facts/inventory-locations.yml");
    let locations = InventoryLocations::parse(unsupported);
    assert_eq!(locations.line("", "NOT-A-SEQUENCE"), None);
    assert_eq!(locations.line("", "IGNORED"), None);
}

#[test]
fn failed_source_read_is_recorded_for_preparsed_documents() {
    use crate::codebase::ci_workflows::{ParsedWorkflowDocument, ParsedWorkflowSet};
    use crate::codebase::ts_source::{FileInventory, SourceStore};

    let root = std::env::temp_dir().join("no-mistakes-secret-facts-missing-root");
    let path = root.join("missing.yml");
    let source = include_str!("../../../fixtures/ci-secret-facts/scanner.yml");
    let value = serde_yaml::from_str(source).unwrap();
    let workflows = ParsedWorkflowSet {
        documents: vec![ParsedWorkflowDocument {
            path: "missing.yml".into(),
            value: Ok(value),
        }],
    };
    let sources = SourceStore::new(Arc::new(FileInventory::from_paths(&[path])));
    let facts = super::collect(
        &root,
        &workflows,
        &ParsedWorkflowSet {
            documents: Vec::new(),
        },
        &sources,
    );
    assert!(facts.workflows["missing.yml"].incomplete[0]
        .1
        .contains("document unavailable"));
}
