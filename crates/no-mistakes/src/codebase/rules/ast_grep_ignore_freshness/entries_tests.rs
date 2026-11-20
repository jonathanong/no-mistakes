use super::ignores_all;
use serde_yaml::Value;

fn entries(source: &str) -> Vec<super::IgnoreEntry> {
    let value: Value = serde_yaml::from_str(source).unwrap();
    ignores_all(std::slice::from_ref(&value), source)
}

#[test]
fn flow_list_reports_each_entry_on_its_line() {
    let source = "id: widgets\nignores: ['src/missing.ts', 'src/items.ts']\n";
    let entries = entries(source);
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].pattern, "src/missing.ts");
    assert_eq!(entries[0].line, 2);
    assert_eq!(entries[1].pattern, "src/items.ts");
    assert_eq!(entries[1].line, 2);
}

#[test]
fn block_list_uses_each_entry_line() {
    let source = "ignores:\n  - src/missing.ts\n  - src/items.ts\n";
    let entries = entries(source);
    assert_eq!(entries[0].line, 2);
    assert_eq!(entries[1].line, 3);
}

#[test]
fn repeated_pattern_uses_the_next_line() {
    let source = "ignores:\n  - src/items.ts\n  - src/items.ts\n";
    let entries = entries(source);
    assert_eq!(entries[0].line, 2);
    assert_eq!(entries[1].line, 3);
}

#[test]
fn empty_glob_skips_glob_lines_already_used_by_patterns() {
    let source = "ignores:\n  - glob: src/items.ts\n  - glob: \"\"\n";
    let entries = entries(source);
    assert_eq!(entries[0].line, 2);
    assert!(!entries[0].invalid_shape);
    assert_eq!(entries[1].line, 3);
    assert!(entries[1].invalid_shape);
}

#[test]
fn later_documents_keep_their_own_lines() {
    let source = "\
id: first
ignores:
  - src/missing.ts
---
id: second
ignores:
  - src/items.ts
";
    let documents = crate::codebase::structured_value::parse_yaml_documents(source);
    assert!(documents.errors.is_empty());
    let entries = ignores_all(&documents.values, source);
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].pattern, "src/missing.ts");
    assert_eq!(entries[0].line, 3);
    assert_eq!(entries[1].pattern, "src/items.ts");
    assert_eq!(entries[1].line, 7);
}
