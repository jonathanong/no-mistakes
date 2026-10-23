use super::*;

#[test]
fn saved_shadowing_scopes_preserve_only_visible_const_findings() {
    let mut failures = Vec::new();
    for case in [
        "catch",
        "declarations",
        "var",
        "loops",
        "switch",
        "parameters",
    ] {
        let path = fixture()
            .with_file_name("shadowing")
            .join(format!("{case}.ts"));
        let source = std::fs::read_to_string(&path).unwrap();
        let expected: Vec<_> = source
            .lines()
            .enumerate()
            .filter_map(|(index, line)| line.contains("// finding").then_some(index + 1))
            .collect();
        let findings = extract(&path, &source).unwrap();
        let actual: Vec<_> = findings.iter().map(|finding| finding.line).collect();
        if actual != expected {
            failures.push(format!("{case}: actual={actual:?}, expected={expected:?}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
