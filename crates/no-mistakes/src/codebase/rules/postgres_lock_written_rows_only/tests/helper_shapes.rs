use super::*;

#[test]
fn unsupported_helpers_never_certify_that_the_caller_writes_no_rows() {
    let root = fixture("helper-unsupported-shapes");
    let path = root.join("src/query.ts");
    let source = std::fs::read_to_string(&path).unwrap();
    let findings = check_with_files(&root, &config(), &[path]).unwrap();
    let lines: Vec<_> = source
        .lines()
        .enumerate()
        .filter_map(|(index, line)| line.contains("FOR UPDATE").then_some(index + 1))
        .collect();
    assert_eq!(lines.len(), 14);
    let missing = source
        .lines()
        .position(|line| line.contains("await missingHelper("))
        .unwrap()
        + 1;
    assert!(
        findings
            .iter()
            .any(|finding| finding.line == missing
                && finding.target.as_deref() == Some("unanalyzable")),
        "{findings:?}"
    );
    for line in lines {
        assert!(
            findings
                .iter()
                .any(|finding| finding.line == line
                    && finding.target.as_deref() == Some("unanalyzable")),
            "line {line}: {findings:?}"
        );
        assert!(!findings.iter().any(|finding| finding.line == line
            && finding.target.as_deref() == Some(CHECKS[0])), "line {line}: {findings:?}");
    }
    assert!(
        !findings
            .iter()
            .any(|finding| finding.target.as_deref() == Some(CHECKS[3])),
        "{findings:?}"
    );
}

#[test]
fn reexports_preserve_keys_and_caller_and_helper_locations() {
    let root = fixture("helper-reexport");
    let files = ["query.ts", "helper.ts", "barrel.ts"].map(|file| root.join("src").join(file));
    let findings = check_with_files(&root, &config(), &files).unwrap();
    assert_eq!(findings.len(), 3, "{findings:?}");
    let mismatch = findings.iter().find(|finding| finding.line == 10).unwrap();
    assert_eq!(mismatch.file, "src/query.ts");
    assert_eq!(mismatch.target.as_deref(), Some(CHECKS[0]));
    assert!(mismatch
        .message
        .contains("helper acquireUser locks at line 3"));
    let unknown = findings.iter().find(|finding| finding.line == 20).unwrap();
    assert_eq!(unknown.target.as_deref(), Some("unanalyzable"));
    assert!(unknown.message.contains("executed SQL cannot be recovered"));
    let definition = findings.iter().find(|finding| finding.line == 9).unwrap();
    assert_eq!(definition.file, "src/helper.ts");
    assert_eq!(definition.target.as_deref(), Some("unanalyzable"));
}
