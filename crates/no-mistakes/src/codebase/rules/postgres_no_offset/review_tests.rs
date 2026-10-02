use super::tests::{config_with_options, fixture};
use super::*;

#[test]
fn non_analyzing_explain_does_not_execute_the_offset_query() {
    let root = fixture("review-followups");
    let path = root.join("db/explain.sql");
    let text = std::fs::read_to_string(&path).unwrap();
    assert_eq!(
        crate::codebase::postgres::sql_file_offset_uses(&text),
        [
            (3, crate::codebase::postgres::OffsetUse::Other),
            (4, crate::codebase::postgres::OffsetUse::Zero),
            (5, crate::codebase::postgres::OffsetUse::Other),
            (7, crate::codebase::postgres::OffsetUse::Other),
        ]
    );
    let findings = check_with_files(
        &root,
        &config_with_options("sqlInclude: ['*.sql']"),
        &[path],
    )
    .unwrap();
    assert_eq!(
        findings
            .iter()
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        [3, 4, 5, 7]
    );
}

#[test]
fn custom_executor_lookup_canonicalizes_duplicate_unsorted_names() {
    let root = fixture("review-followups");
    let config = config_with_options(
        "importSpecifier: '@custom/database'\nexecutorNames: [last, first, last]",
    );
    let path = root.join("src/custom-executors.ts");
    let findings = check_with_files(&root, &config, &[path]).unwrap();
    assert_eq!(
        findings
            .iter()
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        [2, 3]
    );
}

#[test]
fn static_append_keeps_fragment_lines_and_physical_suppressions() {
    let root = fixture("review-followups");
    let path = root.join("src/append-positions.ts");
    let mut findings = check_with_files(
        &root,
        &config_with_options("{}"),
        std::slice::from_ref(&path),
    )
    .unwrap();
    assert_eq!(
        findings
            .iter()
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        [5, 8, 12, 18, 18, 21, 28]
    );
    let sources = super::super::source_store_for_files(std::slice::from_ref(&path));
    super::super::suppress_rule_findings_with_sources(&root, &mut findings, &sources);
    assert_eq!(
        findings
            .iter()
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        [5, 8, 18, 18, 21]
    );
}
