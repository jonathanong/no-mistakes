use super::deferred_tests::SQL;
use super::tests::{config_yaml, fixture};
use super::*;

#[test]
fn multiline_whole_rows_and_lenient_projections_keep_suppression_lines() {
    let root = fixture("deferred");
    for (file, expected) in [("function-lines.sql", 3), ("lenient-lines.sql", 4)] {
        let files = [root.join("sql").join(file)];
        let mut findings = check_with_files(
            &root,
            &config_yaml(&format!(
                "{SQL}allowWholeRowFunctions: []\nunanalyzableSql: ignore\n"
            )),
            &files,
        )
        .unwrap();
        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings[0].line, expected);
        super::super::suppress_rule_findings_with_sources(
            &root,
            &mut findings,
            &super::super::source_store_for_files(&files),
        );
        assert!(findings.is_empty(), "{findings:?}");
    }
}
