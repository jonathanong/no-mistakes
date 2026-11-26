use super::document_spans::document_spans;
use super::parse_yaml_documents;

#[test]
fn document_spans_track_markers_directives_and_bom() {
    let bom_multi = "\u{feff}---\nid: one\n---\nid: two\n";
    let spans = document_spans(bom_multi);
    assert_eq!(spans.len(), 2);
    assert_eq!(spans[0].start_line, 1);
    assert_eq!(spans[1].start_line, 3);

    let with_end_and_tag = "id: one\n...\n# comment\n\n%YAML 1.1\n%TAG !w! tag:example.invalid,2026:a\n---\nid: two\n...\nid: three\n";
    let spans = document_spans(with_end_and_tag);
    assert_eq!(spans.len(), 3);
    assert_eq!(spans[0].start_line, 1);
    assert_eq!(spans[1].start_line, 5);
    assert_eq!(spans[2].start_line, 10);

    let bom_after_end = "id: one\n...\n\u{feff}%TAG !w! tag:example.invalid,2026:a\n---\nid: two\n";
    let spans = document_spans(bom_after_end);
    assert_eq!(spans.len(), 2);
    assert_eq!(spans[0].start_line, 1);
    assert_eq!(spans[1].start_line, 3);
}

#[test]
fn later_document_reparses_after_earlier_document_stops_location_scan() {
    let long_tag = format!("%TAG !w! tag:example.invalid,2026:{}", "a".repeat(1200));
    let source = format!(
        "{long_tag}\n---\nid: first\n---\ndefaults: &defaults\n  owner: widgets\nruntime:\n  <<: *defaults\n"
    );
    let docs = parse_yaml_documents(&source);
    assert!(docs.errors.is_empty(), "{:?}", docs.errors);
    assert_eq!(docs.values.len(), 2);
    assert_eq!(docs.values[1]["runtime"]["owner"].as_str(), Some("widgets"));
}

#[test]
fn unlocated_merge_remains_scoped_inside_a_tagged_sequence_item() {
    let path = crate::test_support::rule_fixture_root("structured-config-policy")
        .parent()
        .unwrap()
        .join("merge-location-unavailable/tagged-child.yml");
    let source = std::fs::read_to_string(&path).unwrap();
    let error = super::parse_structured_value_located(&path, &source).unwrap_err();
    assert!(error.missing_merge_location);
    let partial = error.partial.unwrap();
    let runtime = &partial.value()["runtime"];
    assert!(!partial.mapping_unknown(runtime));
    assert_eq!(runtime["known"].as_str(), Some("local"));
    assert!(!partial.contains_unknown(&runtime["known"]));
    assert!(partial.contains_unknown(&runtime["settings"]));
    let serde_yaml::Value::Tagged(item) = &runtime["settings"][0] else {
        panic!("the item should retain its tag");
    };
    assert!(partial.mapping_unknown(&item.value));
    assert_eq!(item.value["enabled"].as_bool(), Some(true));
    assert!(item.value.get("owner").is_none());
}
