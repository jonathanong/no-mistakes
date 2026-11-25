use super::document_spans::document_spans;
use super::plain_merge_scan::has_plain_merge_key;
use super::{parse_structured_value_located, parse_yaml_documents};
use std::path::Path;

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
  !<tag:yaml.org,2002:str> '<<': keep
  &k \"<<\": keep
pipe_in_word: foo|bar
cmp: a > b
expr: a << b
word: foo<<:bar
literal_block: |-
  <<: *defaults

  still inside block
folded_block: >+2 # comment
  <<: *defaults
note: it's done
items:
  - name: first
    \"<<\": keep
  - \"quoted\": first
    '<<': keep
flow: { note: it's done, \"<<\": keep, '<<': keep, \"a\":\"<<\" }
runtime:
  \"<<\": keep
  '<<': keep
";
    assert!(!has_plain_merge_key(source));
    assert!(!has_plain_merge_key("\u{feff}--- \"<<\": keep\n"));
    assert!(!has_plain_merge_key("? '<<'\n: keep\n"));
    assert!(!has_plain_merge_key("{ ? '<<' : 1 }\n"));
    assert!(!has_plain_merge_key(
        "runtime:\n  a:\n    first\n    'second\n  \"<<\": keep\n"
    ));
    assert!(!has_plain_merge_key("runtime:\n  ?\nother: a << b\n"));
}

#[test]
fn plain_merge_scan_detects_plain_flow_explicit_and_anchored_keys() {
    assert!(has_plain_merge_key("runtime:\n  <<: *defaults\n"));
    assert!(has_plain_merge_key("runtime: {a: 1,<<: *defaults}\n"));
    assert!(has_plain_merge_key("runtime: {<<:{legacy: true}}\n"));
    assert!(has_plain_merge_key("runtime: {<<:[*defaults]}\n"));
    assert!(has_plain_merge_key(
        "runtime: {\n  << # merge\n  : *defaults\n}\n"
    ));
    assert!(has_plain_merge_key("? <<\n: *defaults\n"));
    assert!(has_plain_merge_key(
        "runtime:\n  ?\n    <<\n  : *defaults\n"
    ));
    assert!(has_plain_merge_key("{\n  ?\n    <<\n  : *defaults\n}\n"));
    assert!(has_plain_merge_key(
        "key: &k <<\nruntime:\n  *k: *defaults\n"
    ));
    assert!(has_plain_merge_key(
        "key: &k\n  <<\nruntime:\n  *k: *defaults\n"
    ));
    assert!(has_plain_merge_key("runtime: !custom\n  <<: *defaults\n"));
    assert!(has_plain_merge_key("items:\n  - <<: *defaults\n"));
    assert!(!has_plain_merge_key("runtime:\n  <<:not_a_key\n"));
}

#[test]
fn plain_merge_scan_does_not_let_mid_scalar_quotes_hide_merge_keys() {
    assert!(has_plain_merge_key(
        "note: it's done\nruntime:\n  <<: *defaults\n"
    ));
    assert!(has_plain_merge_key(
        "note: it 's done\nruntime:\n  <<: *defaults\n"
    ));
    assert!(has_plain_merge_key(
        "note: say \"hi\nruntime:\n  <<: *defaults\n"
    ));
    assert!(has_plain_merge_key(
        "note: line one\n  'continuation\nruntime:\n  <<: *defaults\n"
    ));
    assert!(has_plain_merge_key(
        "note:\n  hello\n  'world\nruntime:\n  <<: *defaults\nother: 'later'\n"
    ));
    assert!(has_plain_merge_key(
        "outer:\n  note:\n    hello\n    'world\n  runtime:\n    <<: *defaults\nother: 'later'\n"
    ));
    assert!(has_plain_merge_key(
        "items:\n  -\n    hello\n    'world\n  - <<: *defaults\nother: 'later'\n"
    ));
    assert!(has_plain_merge_key(
        "note: hello, 'world\nruntime:\n  <<: *defaults\n"
    ));
    assert!(has_plain_merge_key(
        "meta: { note: it's done, <<: *defaults }\n"
    ));
    assert!(has_plain_merge_key(
        "items:\n  - note: compare a >\n    <<: *defaults\n"
    ));
    assert!(has_plain_merge_key(
        "items:\n  - text: |\n      <<: *ignored\n    <<: *defaults\n"
    ));
    assert!(has_plain_merge_key(
        "note: <<'s done\nruntime:\n  <<: *defaults\n"
    ));
    assert!(has_plain_merge_key("'unclosed\n<<: *defaults\n"));
    assert!(has_plain_merge_key("\"unclosed\n<<: *defaults\n"));
    assert!(has_plain_merge_key(
        "tag: !<unterminated\nruntime:\n  <<: *defaults\n"
    ));
    assert!(has_plain_merge_key(
        "note: first\n\n  # comment\nruntime:\n  <<: *defaults\n"
    ));
}

#[test]
fn unlocated_merge_fixtures_after_apostrophe_flow_and_explicit_keys_are_reported() {
    let expected =
        "failed to apply YAML merge keys (could not match a YAML `<<` key to its source location)";
    let fixtures = [
        (
            "long-tag-apostrophe.yml",
            include_str!(
                "../../../../../test-cases/rules/structured-config-policy/merge-location-unavailable/long-tag-apostrophe.yml"
            ),
        ),
        (
            "long-tag-multiline-scalar.yml",
            include_str!(
                "../../../../../test-cases/rules/structured-config-policy/merge-location-unavailable/long-tag-multiline-scalar.yml"
            ),
        ),
        (
            "long-tag-flow-merge.yml",
            include_str!(
                "../../../../../test-cases/rules/structured-config-policy/merge-location-unavailable/long-tag-flow-merge.yml"
            ),
        ),
        (
            "long-tag-explicit-merge.yml",
            include_str!(
                "../../../../../test-cases/rules/structured-config-policy/merge-location-unavailable/long-tag-explicit-merge.yml"
            ),
        ),
    ];
    for (name, source) in fixtures {
        let error = parse_structured_value_located(Path::new(name), source).unwrap_err();
        assert_eq!(error, (Some(1), expected.to_string()), "{name}");

        let documents = parse_yaml_documents(source);
        assert!(documents.values.is_empty(), "{name}");
        assert_eq!(documents.errors, vec![(1, expected.to_string())], "{name}");
    }
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
