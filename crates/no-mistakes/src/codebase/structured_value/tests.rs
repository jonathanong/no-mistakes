use super::{is_jsonc_path, parse_structured_value, parse_yaml_documents};
use serde_yaml::Value;
use std::path::Path;

#[test]
fn parses_yaml_mappings() {
    let value = parse_structured_value(Path::new("app.yml"), "runtime:\n  version: 1\n").unwrap();
    assert_eq!(value["runtime"]["version"].as_i64(), Some(1));
}

#[test]
fn parses_jsonc_comments_and_trailing_commas() {
    let source = "{\n  // keep plugins\n  \"plugins\": [\"react\"],\n}\n";
    let value = parse_structured_value(Path::new(".oxlintrc.json"), source).unwrap();
    let plugins = value.get("plugins").and_then(Value::as_sequence).unwrap();
    assert_eq!(plugins[0].as_str(), Some("react"));
}

#[test]
fn jsonc_parse_errors_are_diagnostics() {
    let error = parse_structured_value(Path::new("broken.json"), "{").unwrap_err();
    assert!(error.contains("failed to parse JSONC"), "{error}");
}

#[test]
fn yaml_documents_keep_each_mapping() {
    let parsed = parse_yaml_documents("id: one\n---\nid: two\n");
    assert!(parsed.errors.is_empty());
    assert_eq!(parsed.values.len(), 2);
    assert_eq!(parsed.values[0]["id"].as_str(), Some("one"));
    assert_eq!(parsed.values[1]["id"].as_str(), Some("two"));
}

#[test]
fn yaml_documents_keep_a_valid_document_before_a_broken_one() {
    let parsed = parse_yaml_documents("id: widgets\n---\nignores: [\n---\nid: orders\n");
    assert_eq!(parsed.values.len(), 1);
    assert_eq!(parsed.values[0]["id"].as_str(), Some("widgets"));
    assert_eq!(parsed.errors.len(), 1);
    assert!(parsed.errors[0].1.contains("failed to parse YAML"));
}

#[test]
fn yaml_documents_stop_when_a_syntax_error_ends_the_file() {
    let parsed = parse_yaml_documents("id: widgets\n---\nignores: [\n");
    assert_eq!(parsed.values.len(), 1);
    assert_eq!(parsed.values[0]["id"].as_str(), Some("widgets"));
    assert_eq!(parsed.errors.len(), 1);
}

#[test]
fn yaml_documents_keep_an_error_on_the_last_source_line() {
    let parsed = parse_yaml_documents("ignores: [\n");
    assert_eq!(parsed.errors.len(), 1);
    assert_eq!(parsed.errors[0].0, 1);
}

#[test]
fn single_document_parser_rejects_multiple_documents() {
    let error =
        parse_structured_value(Path::new("rules.yml"), "id: one\n---\nid: two\n").unwrap_err();
    assert!(error.contains("failed to parse YAML"), "{error}");
}

#[test]
fn yaml_parse_errors_are_diagnostics() {
    let error =
        parse_structured_value(Path::new("broken.yml"), "runtime:\n  version: [\n").unwrap_err();
    assert!(error.contains("failed to parse YAML"), "{error}");
}

#[test]
fn jsonc_path_detection_covers_json_extensions() {
    assert!(is_jsonc_path(Path::new("a.json")));
    assert!(is_jsonc_path(Path::new("a.JSONC")));
    assert!(!is_jsonc_path(Path::new("a.yml")));
}

#[test]
fn converts_json_number_shapes() {
    let source = r#"{ "i": 2, "u": 1, "f": 1.5, "ok": true, "empty": null }"#;
    let value = parse_structured_value(Path::new("n.json"), source).unwrap();
    assert_eq!(value["i"].as_i64(), Some(2));
    assert!(value["ok"].as_bool().unwrap());
    assert!(value["empty"].is_null());
    assert!(value["f"].as_f64().unwrap() > 1.0);
}

#[test]
fn expands_a_mapping_merge_key() {
    let source = "defaults: &defaults\n  legacy:\n    enabled: true\nruntime:\n  <<: *defaults\n  version: 2\n";
    let value = parse_structured_value(Path::new("app.yml"), source).unwrap();
    assert_eq!(value["runtime"]["legacy"]["enabled"].as_bool(), Some(true));
    assert_eq!(value["runtime"]["version"].as_i64(), Some(2));
    assert!(value["runtime"].get("<<").is_none());
}

#[test]
fn sequence_merge_keeps_the_earlier_mapping() {
    let source = "\
runtime:
  version: 2
  <<:
    - legacy: { enabled: true }
      owner: first
    - legacy: { enabled: false }
      owner: second
      extra: 1
";
    let value = parse_structured_value(Path::new("app.yml"), source).unwrap();
    assert_eq!(value["runtime"]["version"].as_i64(), Some(2));
    assert_eq!(value["runtime"]["legacy"]["enabled"].as_bool(), Some(true));
    assert_eq!(value["runtime"]["owner"].as_str(), Some("first"));
    assert_eq!(value["runtime"]["extra"].as_i64(), Some(1));
}

#[test]
fn sequence_merge_with_a_scalar_item_is_an_error() {
    let source = "\
runtime:
  <<:
    - legacy: { enabled: true }
    - plain
";
    let error = parse_structured_value(Path::new("app.yml"), source).unwrap_err();
    assert!(error.contains("failed to apply YAML merge keys"), "{error}");
    assert!(error.contains("found scalar"), "{error}");
}

#[test]
fn scalar_merge_is_an_error_instead_of_being_dropped() {
    let source = "label: &label plain\nruntime:\n  <<: *label\n  version: 1\n";
    let error = parse_structured_value(Path::new("app.yml"), source).unwrap_err();
    assert!(error.contains("failed to apply YAML merge keys"), "{error}");
    let empty = "runtime:\n  <<: 7\n";
    assert!(parse_structured_value(Path::new("app.yml"), empty).is_err());
}

#[test]
fn nested_merges_inside_a_merged_mapping_are_expanded() {
    let source = "\
inner: &inner
  owner: inner
  depth: 3
middle: &middle
  <<: *inner
  owner: middle
runtime:
  <<: *middle
  version: 2
";
    let value = parse_structured_value(Path::new("app.yml"), source).unwrap();
    assert_eq!(value["runtime"]["version"].as_i64(), Some(2));
    assert_eq!(value["runtime"]["owner"].as_str(), Some("middle"));
    assert_eq!(value["runtime"]["depth"].as_i64(), Some(3));
    assert!(value["runtime"].get("<<").is_none());
}

#[test]
fn merges_inside_sequence_sources_and_values_are_expanded() {
    let source = "\
base: &base
  deep:
    <<: &leaf { flag: true }
list:
  - <<: *base
";
    let value = parse_structured_value(Path::new("app.yml"), source).unwrap();
    assert_eq!(value["list"][0]["deep"]["flag"].as_bool(), Some(true));
}

#[test]
fn tagged_values_are_searched_for_merge_keys() {
    let source = "runtime: !custom\n  <<: { legacy: true }\n";
    let value = parse_structured_value(Path::new("app.yml"), source).unwrap();
    let Value::Tagged(tagged) = &value["runtime"] else {
        panic!("runtime keeps its tag");
    };
    assert_eq!(tagged.value["legacy"].as_bool(), Some(true));
    let bad = "runtime: !custom\n  <<: 7\n";
    assert!(parse_structured_value(Path::new("app.yml"), bad).is_err());
    let tagged_merge = "runtime:\n  <<: !custom { legacy: true }\n";
    assert!(parse_structured_value(Path::new("app.yml"), tagged_merge).is_err());
}

#[test]
fn yaml_documents_report_a_merge_that_cannot_be_applied() {
    let source = "ok: 1\n---\nruntime:\n  <<: 7\n---\nlast: 2\n";
    let documents = parse_yaml_documents(source);
    assert_eq!(documents.values.len(), 2);
    assert_eq!(documents.errors.len(), 1);
    assert_eq!(documents.errors[0].0, 4);
    assert!(documents.errors[0].1.contains("merge keys"));
    let flow = "ok: 1\n---\n{ runtime: { <<: 7 } }\n";
    assert_eq!(parse_yaml_documents(flow).errors[0].0, 3);
    let second_doc = "---\na: &a\n  x: 1\nb:\n  <<: *a\n---\n# <<: 7\nnote: \"escaped \\\" <<: 7\"\nitems:\n  - <<: 7\n";
    assert_eq!(parse_yaml_documents(second_doc).errors[0].0, 10);
    let second_merge = "a: &a\n  x: 1\ngood:\n  <<: *a\nbad:\n  <<: 7\n";
    assert_eq!(parse_yaml_documents(second_merge).errors[0].0, 6);
}

#[test]
fn json_keeps_a_literal_merge_key() {
    let value =
        parse_structured_value(Path::new("app.json"), r#"{ "<<": { "legacy": 1 } }"#).unwrap();
    assert!(value.get("<<").is_some());
}

#[test]
fn repeated_yaml_syntax_error_is_reported_once() {
    let source = include_str!(
        "../../../../../test-cases/rules/ast-grep-ignore-freshness/fixture/tracked/rules/repeated-doc.yml"
    );
    let parsed = parse_yaml_documents(source);
    assert_eq!(parsed.errors.len(), 1);
    assert_eq!(parsed.values[0]["id"].as_str(), Some("widgets"));
}

#[test]
fn converts_json_integers_that_do_not_fit_i64() {
    let source = r#"{ "u": 18446744073709551615 }"#;
    let value = parse_structured_value(Path::new("n.json"), source).unwrap();
    assert!(value["u"].as_i64().is_none());
    assert_eq!(value["u"].as_u64(), Some(u64::MAX));
}

#[test]
fn saved_merge_key_edge_fixtures_report_expected_errors_and_lines() {
    let nested_list = include_str!(
        "../../../../../test-cases/structured-value/merge-keys/sequence-nested-list.yml"
    );
    let error =
        parse_structured_value(Path::new("sequence-nested-list.yml"), nested_list).unwrap_err();
    assert!(
        error.contains("expected a mapping for merging, but found sequence"),
        "{error}"
    );
    let nested_docs = parse_yaml_documents(nested_list);
    assert_eq!(nested_docs.errors.len(), 1);
    assert_eq!(nested_docs.errors[0].0, 2);

    let quoted_fallback = include_str!(
        "../../../../../test-cases/structured-value/merge-keys/quoted-key-fallback.yml"
    );
    let quoted_docs = parse_yaml_documents(quoted_fallback);
    assert!(quoted_docs.errors.is_empty());
    assert_eq!(quoted_docs.values.len(), 2);
    assert_eq!(quoted_docs.values[1]["runtime"]["<<"].as_i64(), Some(7));

    let aliased_fallback = include_str!(
        "../../../../../test-cases/structured-value/merge-keys/aliased-merge-fallback.yml"
    );
    let aliased_docs = parse_yaml_documents(aliased_fallback);
    assert!(aliased_docs.values.is_empty());
    assert_eq!(aliased_docs.errors.len(), 1);
    assert_eq!(aliased_docs.errors[0].0, 7);

    let compact_flow = include_str!(
        "../../../../../test-cases/structured-value/merge-keys/compact-flow-and-false-tokens.yml"
    );
    let compact_docs = parse_yaml_documents(compact_flow);
    assert_eq!(compact_docs.values.len(), 2);
    assert_eq!(compact_docs.errors.len(), 1);
    assert_eq!(compact_docs.errors[0].0, 8);

    let compact_brace = include_str!(
        "../../../../../test-cases/structured-value/merge-keys/compact-brace-flow.yml"
    );
    let brace_docs = parse_yaml_documents(compact_brace);
    assert_eq!(brace_docs.errors.len(), 1);
    assert_eq!(brace_docs.errors[0].0, 1);
}

#[test]
fn reports_the_original_merge_key_with_aliases_and_flow_syntax() {
    let source = include_str!(
        "../../../../../test-cases/rules/structured-config-policy/merge-key/aliased-location.yml"
    );
    let error = parse_structured_value(Path::new("app.yml"), source).unwrap_err();
    assert!(error.ends_with("at line 4"), "{error}");
    let documents = parse_yaml_documents(source);
    assert!(documents.values.is_empty());
    assert_eq!(documents.errors.len(), 1);
    assert_eq!(documents.errors[0].0, 4);
}

#[test]
fn earlier_sequence_source_wins_after_its_nested_merge() {
    let source = include_str!(
        "../../../../../test-cases/structured-value/merge-keys/sequence-precedence.yml"
    );
    let value = parse_structured_value(Path::new("app.yml"), source).unwrap();
    let result = &value["result"];
    assert_eq!(result["color"].as_str(), Some("green"));
    assert_eq!(result["size"].as_str(), Some("small"));
    assert_eq!(result["tier"].as_str(), Some("basic"));
}

#[test]
fn reports_the_key_that_survives_merge_precedence() {
    let source = include_str!(
        "../../../../../test-cases/structured-value/merge-keys/overridden-location.yml"
    );
    let error = parse_structured_value(Path::new("app.yml"), source).unwrap_err();
    assert!(error.ends_with("at line 4"), "{error}");
    let documents = parse_yaml_documents(source);
    assert_eq!(documents.errors.len(), 1);
    assert_eq!(documents.errors[0].0, 4);
}

#[test]
fn reports_a_failed_merge_inside_an_earlier_sequence_source() {
    let source = include_str!(
        "../../../../../test-cases/structured-value/merge-keys/nested-source-error.yml"
    );
    let error = parse_structured_value(Path::new("app.yml"), source).unwrap_err();
    assert!(error.ends_with("at line 4"), "{error}");
}

#[test]
fn detects_a_merge_inside_a_tagged_mapping() {
    let source = include_str!(
        "../../../../../test-cases/structured-value/merge-keys/tagged-merge-error.yml"
    );
    let value = serde_yaml::from_str(source).unwrap();
    let error = super::expand_yaml_mapping_if_merged(&value, Some(source)).unwrap_err();
    assert!(error.contains("failed to apply YAML merge keys"), "{error}");
}

#[test]
fn locates_a_merge_error_before_a_later_yaml_syntax_error() {
    let source = include_str!(
        "../../../../../test-cases/structured-value/merge-keys/merge-before-syntax-error.yml"
    );
    let documents = parse_yaml_documents(source);
    assert_eq!(documents.values.len(), 1);
    assert_eq!(documents.errors.len(), 2);
    assert_eq!(documents.errors[0].0, 5);
    assert!(documents.errors[0]
        .1
        .contains("failed to apply YAML merge keys"));
    assert!(documents.errors[1].1.contains("failed to parse YAML"));
}

#[test]
fn workflow_merge_supports_values_without_source_text() {
    let source = include_str!(
        "../../../../../test-cases/rules/workflow-topology-policy/fixture/valid-nested-merges/.github/workflows/job.yml"
    );
    let value = serde_yaml::from_str(source).unwrap();
    let expanded = super::expand_yaml_mapping_if_merged(&value, None).unwrap();
    assert_eq!(
        expanded["jobs"]["verify"]["timeout-minutes"].as_i64(),
        Some(8)
    );

    let source = include_str!(
        "../../../../../test-cases/rules/workflow-topology-policy/fixture/merge-keys/.github/workflows/job-only.yml"
    );
    let value = serde_yaml::from_str(source).unwrap();
    let error = super::expand_yaml_mapping_if_merged(&value, None).unwrap_err();
    assert!(error.contains("failed to apply YAML merge keys"), "{error}");
}

#[test]
fn merge_keys_without_source_locations_invalidate_the_document() {
    let source = include_str!(
        "../../../../../test-cases/rules/structured-config-policy/merge-location-unavailable/long-tag.yml"
    );
    let value: Value = serde_yaml::from_str(source).unwrap();
    assert_eq!(value["runtime"]["<<"]["legacy"].as_bool(), Some(true));

    let expected =
        "failed to apply YAML merge keys (could not match a YAML `<<` key to its source location)";
    let error =
        super::parse_structured_value_located(Path::new("long-tag.yml"), source).unwrap_err();
    assert_eq!(error, (Some(1), expected.to_string()));

    let documents = parse_yaml_documents(source);
    assert!(documents.values.is_empty());
    assert_eq!(documents.errors, vec![(1, expected.to_string())]);

    let error = super::expand_yaml_mapping_if_merged(&value, Some(source)).unwrap_err();
    assert_eq!(error, expected);
}

#[test]
fn merge_key_must_match_the_corresponding_source_key() {
    let source = include_str!(
        "../../../../../test-cases/rules/structured-config-policy/merge-location-unavailable/quoted.yml"
    );
    let mut value: Value = serde_yaml::from_str(source).unwrap();
    let defaults = value["defaults"].as_mapping_mut().unwrap();
    let owner = defaults.shift_remove("owner").unwrap();
    defaults.insert(Value::String("<<".to_string()), owner);

    let error = super::expand_yaml_mapping_if_merged(&value, Some(source)).unwrap_err();
    assert!(error.contains("could not match a YAML `<<` key to its source location"));
}

#[test]
fn missing_location_in_a_nested_merge_source_is_an_error() {
    use super::merge_keys::{expand_located, KeyLocation};

    let source = include_str!(
        "../../../../../test-cases/structured-value/merge-keys/nested-source-error.yml"
    );
    let mut value: Value = serde_yaml::from_str(source).unwrap();
    let outer_key = value["runtime"]
        .as_mapping()
        .unwrap()
        .keys()
        .find_map(Value::as_str)
        .unwrap()
        .as_ptr();
    let error = expand_located(&mut value, &|key| {
        if key.as_ptr() == outer_key {
            Ok(KeyLocation::Merge(2))
        } else {
            Err((1, "missing source location".to_string()))
        }
    })
    .unwrap_err();
    assert_eq!(error, (1, "missing source location".to_string()));
}
