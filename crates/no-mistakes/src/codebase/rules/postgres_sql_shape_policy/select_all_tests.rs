use super::shape_tests::{config_yaml, fixture};
use super::*;

#[test]
fn select_all_keyset_pages_keep_findings_but_distinct_pages_are_excluded() {
    let root = fixture("select-all");
    let config = config_yaml("sqlInclude: ['sql/**/*.sql']\nbannedShapes: [keyset-only-sweep]\n");
    let findings = check_with_files(&root, &config, &[root.join("sql/pages.sql")]).unwrap();
    assert_eq!(
        findings
            .iter()
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        [2, 3, 6, 7],
        "{findings:#?}"
    );
}
