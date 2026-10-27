use super::locate::set_line;
use super::SetSpec;

fn spec(kind: &str, key: &str, target: &str) -> SetSpec {
    SetSpec {
        kind: kind.to_string(),
        key: key.to_string(),
        target: target.to_string(),
        ..Default::default()
    }
}

const DEPENDABOT: &str = "version: 2\nupdates:\n  - package-ecosystem: npm\n    directory: /\n    cooldown:\n      default-days: 7\n  - package-ecosystem: cargo\n    cooldown:\n      exclude:\n        - a\n";

#[test]
fn yaml_key_line_follows_indexed_sequence_items() {
    let line = |key| set_line(&spec("yaml-sequence", key, ""), DEPENDABOT);
    assert_eq!(line("updates.1.cooldown.exclude"), 9);
    assert_eq!(line("updates.1.cooldown"), 8);
    assert_eq!(line("updates.0.cooldown"), 5);
    // Missing keys fall back to the nearest ancestor that was found.
    assert_eq!(line("updates.0.cooldown.exclude"), 5);
    assert_eq!(line("updates.5.cooldown"), 2);
    assert_eq!(line("missing"), 1);
}

#[test]
fn yaml_selector_line_skips_each_segments() {
    let selector = spec("yaml-string-selector", "updates.[].cooldown.exclude", "");
    // `[]` visits every item, so only the sequence key itself is exact.
    assert_eq!(set_line(&selector, DEPENDABOT), 2);
    let selector = spec("yaml-string-selector", "updates.1.cooldown.exclude", "");
    assert_eq!(set_line(&selector, DEPENDABOT), 9);
    assert_eq!(
        set_line(&spec("yaml-string-selector", "", ""), DEPENDABOT),
        1
    );
}

#[test]
fn yaml_key_line_handles_json_and_unindented_lists() {
    let json = "{\n  \"a\": {\n    \"b\": [\n      \"x\"\n    ]\n  }\n}\n";
    assert_eq!(set_line(&spec("yaml-sequence", "a.b", ""), json), 3);
    let flat = "# c\nitems:\n- one\n- two\nother:\n  - three\n";
    assert_eq!(set_line(&spec("yaml-sequence", "items", ""), flat), 2);
    assert_eq!(set_line(&spec("yaml-sequence", "items.1", ""), flat), 4);
    assert_eq!(set_line(&spec("yaml-sequence", "q", ""), "'q':\n- a\n"), 1);
}

#[test]
fn declaration_line_uses_declarations_only() {
    let source = "// Kind mentioned\nimport { Kind } from './x';\nexport type Kind = 'a';\n";
    assert_eq!(set_line(&spec("ts-string-union", "", "Kind"), source), 3);
    // A bare word is not a declaration, so there is nothing to point at.
    assert_eq!(set_line(&spec("ts-array-literal", "", "Other"), source), 1);
    assert_eq!(set_line(&spec("ts-array-literal", "", ""), source), 1);
    assert_eq!(
        set_line(&spec("markdown-table-code-cells", "", "Kind"), source),
        1
    );
}

#[test]
fn declaration_line_ignores_comments() {
    let line = "// legacy type Kind\n/* const X = 1;\n   type Kind */\nexport type Kind = 'a';\n";
    assert_eq!(set_line(&spec("ts-string-union", "", "Kind"), line), 4);
    let block = "/* const X = [] */\nconst X = ['a'];\n";
    assert_eq!(set_line(&spec("ts-array-literal", "", "X"), block), 2);
}

#[test]
fn sql_declaration_line_is_case_insensitive_and_skips_comments() {
    let sql = "-- CREATE TYPE status AS ENUM ('x');\n\ncreate type status as enum ('a');\n";
    assert_eq!(set_line(&spec("sql-enum", "", "status"), sql), 3);
    let upper = "SELECT 1;\nCREATE TYPE status AS ENUM ('a');\n";
    assert_eq!(set_line(&spec("sql-enum", "", "status"), upper), 2);
}

fn yaml(key: &str, source: &str) -> usize {
    set_line(&spec("yaml-sequence", key, ""), source)
}

#[test]
fn yaml_first_segment_matches_top_level_only() {
    assert_eq!(yaml("a", "x:\n  a:\n    - 1\na:\n  - 2"), 4);
}

#[test]
fn yaml_nested_segment_matches_direct_children_only() {
    let source = "c:\n  d:\n    e: 1\n  e: 2\n";
    assert_eq!(yaml("c.e", source), 4);
}

#[test]
fn yaml_missing_leaf_falls_back_to_its_parent_not_a_sibling() {
    let source = "l:\n  - a:\n      x: 1\n    b:\n      y: 2\n";
    assert_eq!(yaml("l.0.a.y", source), 2);
    assert_eq!(yaml("l.0.b.y", source), 5);
}

#[test]
fn yaml_block_scalar_bodies_are_not_keys() {
    let source = "run: |\n  a: not a key\n  more\nnote: >-\n  b: nope\na:\n  - 1\n";
    assert_eq!(yaml("a", source), 6);
    assert_eq!(yaml("b", source), 1);
}

#[test]
fn yaml_sequence_items_stay_inside_their_own_item() {
    let source = "u:\n  - n: 1\n    k:\n      - a\n  - n: 2\n    j: 1\n";
    assert_eq!(yaml("u.0.j", source), 2);
    assert_eq!(yaml("u.1.j", source), 6);
    assert_eq!(yaml("u.1.n", source), 5);
}

// `- - k` puts two markers on one line; each inner item needs its own line.
const NESTED_SEQUENCES: &str = "m:\n  - - k: 1\n      v: a\n    - k: 2\n      v: b\n  - - k: 3\n";

#[test]
fn yaml_nested_sequence_items_resolve_to_their_own_line() {
    assert_eq!(yaml("m.0.0", NESTED_SEQUENCES), 2);
    assert_eq!(yaml("m.0.1", NESTED_SEQUENCES), 4);
    assert_eq!(yaml("m.0.1.v", NESTED_SEQUENCES), 5);
    assert_eq!(yaml("m.0.0.v", NESTED_SEQUENCES), 3);
    assert_eq!(yaml("m.1.0.k", NESTED_SEQUENCES), 6);
    // A key cannot be read through an unopened inner sequence.
    assert_eq!(yaml("m.0.k", NESTED_SEQUENCES), 2);
    assert_eq!(yaml("m.0.2", NESTED_SEQUENCES), 2);
}

// Pretty-printed and compact arrays must both resolve by index.
const JSON_ARRAYS: &str = "{\n  \"a\": [\n    {\n      \"k\": 1\n    },\n    {\n      \"k\": 2\n    }\n  ],\n  \"b\": [{\"k\": 1}, {\"k\": 2}]\n}\n";
const JSON_TOP_ARRAY: &str = "[\n  {\n    \"k\": 1\n  },\n  {\n    \"k\": 2\n  }\n]\n";

#[test]
fn json_numeric_segments_resolve_array_elements() {
    assert_eq!(yaml("a.1.k", JSON_ARRAYS), 7);
    assert_eq!(yaml("a.0.k", JSON_ARRAYS), 4);
    assert_eq!(yaml("a.1", JSON_ARRAYS), 6);
    assert_eq!(yaml("b.1.k", JSON_ARRAYS), 10);
    // Out-of-range and non-numeric segments fall back to the ancestor.
    assert_eq!(yaml("a.2.k", JSON_ARRAYS), 2);
    assert_eq!(yaml("a.x", JSON_ARRAYS), 2);
    assert_eq!(yaml("0.k", JSON_TOP_ARRAY), 3);
    assert_eq!(yaml("1.k", JSON_TOP_ARRAY), 6);
}

#[test]
fn json_keys_match_without_space_after_colon() {
    let source = "{\n\"a\":[1],\n\"b\":{\"c\":[2]}}\n";
    assert_eq!(yaml("a", source), 2);
    assert_eq!(yaml("b.c", source), 3);
    // Truncated JSON keeps what resolved; unquoted keys use the line scan.
    assert_eq!(yaml("a", "{\n  \"a\": [1,\n"), 2);
    assert_eq!(yaml("a", "{\n  a: [1]\n}\n"), 2);
}

#[test]
fn yaml_block_scalar_header_may_carry_a_comment() {
    let source = "run: | # note\n  a: not a key\nb: >- # c\n  c: nope\na:\n  - 1\n";
    assert_eq!(yaml("a", source), 5);
    assert_eq!(yaml("c", source), 1);
}

#[test]
fn declaration_line_ignores_string_contents() {
    let ts = |source| set_line(&spec("ts-string-union", "", "K"), source);
    let quoted = "const n = 'type K = 1';\nconst t = `\n  type K\n`;\nconst u = \"type K\";\ntype K = 'a';\n";
    assert_eq!(ts(quoted), 6);
    // An unterminated quote ends at the newline and cannot hide the real one.
    assert_eq!(ts("const s = \"don't;\nexport type K = 'a';\n"), 2);
    assert_eq!(ts("const s = 'it\\'s type K';\ntype K = 'a';\n"), 2);
    // Backslash-newline continues a quoted string onto the next line.
    assert_eq!(ts("const s = 'a \\\n type K';\ntype K = 'a';\n"), 3);
}
