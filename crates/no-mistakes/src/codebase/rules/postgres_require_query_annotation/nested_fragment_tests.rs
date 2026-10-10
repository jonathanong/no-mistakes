use super::tests::{config_with_options, fixture, ts_file};
use super::*;

#[test]
fn nested_fragments_do_not_become_annotation_bind_placeholders() {
    let root = fixture("nested-fragments");
    let file = ts_file(&root);
    for (mode, count) in [("report", 5), ("ignore", 1)] {
        let config = config_with_options(&format!(
            "importSpecifier: '@example/db'\ntrustedSqlTags: [{{module: '@example/db', name: sql}}]\nunanalyzableSql: {mode}"
        ));
        let findings = check_with_files(&root, &config, std::slice::from_ref(&file)).unwrap();
        assert_eq!(findings.len(), count, "{mode}: {findings:#?}");
        assert_eq!(
            findings
                .iter()
                .filter(|finding| finding.message.contains("leading SQL is unanalyzable"))
                .count(),
            count - 1
        );
        assert!(findings
            .iter()
            .any(|finding| finding.line == 6 && finding.message.contains("must start")));
    }
}
