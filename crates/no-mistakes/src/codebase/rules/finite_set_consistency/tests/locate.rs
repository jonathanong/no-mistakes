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
    // `[]` visits every item; the first item's nearest found ancestor wins.
    assert_eq!(set_line(&selector, DEPENDABOT), 5);
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
fn declaration_line_prefers_declarations_then_words() {
    let source = "// Kind mentioned\nimport { Kind } from './x';\nexport type Kind = 'a';\n";
    assert_eq!(set_line(&spec("ts-string-union", "", "Kind"), source), 3);
    assert_eq!(set_line(&spec("ts-array-literal", "", "Other"), source), 1);
    assert_eq!(set_line(&spec("ts-array-literal", "", ""), source), 1);
    assert_eq!(
        set_line(&spec("markdown-table-code-cells", "", "Kind"), source),
        1
    );
}
