use super::document_spans::document_spans;
use super::parse_yaml_documents;
use super::plain_merge_scan::has_plain_merge_key;

#[test]
fn plain_merge_scan_ignores_quotes_comments_tags_and_block_scalars() {
    let source = "\
%TAG !w! tag:example.invalid,2026:a
---
# <<: *defaults
double: \"escaped \\\" <<: *defaults\"
single: 'escaped '' <<: *defaults'
tagged:
  !!str <<: keep
pipe_in_word: foo|bar
cmp: a > b
expr: a << b
word: foo<<:bar
literal_block: |-
  <<: *defaults

  still inside block
folded_block: >+2 # comment
  <<: *defaults
runtime:
  \"<<\": keep
  '<<': keep
";
    assert!(!has_plain_merge_key(source));
}

#[test]
fn plain_merge_scan_detects_plain_flow_explicit_and_anchored_keys() {
    assert!(has_plain_merge_key("runtime:\n  <<: *defaults\n"));
    assert!(has_plain_merge_key("runtime: {a: 1,<<: *defaults}\n"));
    assert!(has_plain_merge_key("? <<\n: *defaults\n"));
    assert!(has_plain_merge_key(
        "key: &k <<\nruntime:\n  *k: *defaults\n"
    ));
    assert!(has_plain_merge_key("items:\n  - <<: *defaults\n"));
    assert!(!has_plain_merge_key("runtime:\n  <<:not_a_key\n"));
}

#[test]
fn document_spans_track_markers_directives_and_bom() {
    let bom_multi = "\u{feff}---\nid: one\n---\nid: two\n";
    let spans = document_spans(bom_multi);
    assert_eq!(spans.len(), 2);
    assert_eq!(spans[0].start_line, 1);
    assert_eq!(spans[1].start_line, 3);

    let with_end_and_tag =
        "id: one\n...\n# comment\n\n%YAML 1.1\n%TAG !w! tag:example.invalid,2026:a\n---\nid: two\n...\nid: three\n";
    let spans = document_spans(with_end_and_tag);
    assert_eq!(spans.len(), 3);
    assert_eq!(spans[0].start_line, 1);
    assert_eq!(spans[1].start_line, 5);
    assert_eq!(spans[2].start_line, 10);
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
