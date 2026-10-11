use super::tests::{config_with_options, fixture, ts_file};
use super::*;

#[test]
fn nested_fragments_do_not_become_annotation_bind_placeholders() {
    let root = fixture("nested-fragments");
    let file = ts_file(&root);
    for (mode, count) in [("report", 11), ("ignore", 2)] {
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
            if mode == "report" { 9 } else { 0 }
        );
        assert!(findings
            .iter()
            .any(|finding| finding.line == 6 && finding.message.contains("must start")));
    }
}

#[test]
fn scalar_fragment_operands_stay_binds_and_runtime_candidates_stay_opaque() {
    let root = fixture("scalar-fragment-values");
    let file = ts_file(&root);
    let source = std::fs::read_to_string(&file).unwrap();
    for mode in ["report", "ignore"] {
        let config = config_with_options(&format!(
            "importSpecifier: '@example/db'\ntrustedSqlTags: [{{module: '@example/db', name: sql}}]\nunanalyzableSql: {mode}"
        ));
        let findings = check_with_files(&root, &config, std::slice::from_ref(&file)).unwrap();
        let expected = source
            .lines()
            .enumerate()
            .filter_map(|(index, line)| {
                (line.contains("// missing") || (mode == "report" && line.contains("// unknown")))
                    .then_some(index + 1)
            })
            .collect::<Vec<_>>();
        assert_eq!(
            findings
                .iter()
                .map(|finding| finding.line)
                .collect::<Vec<_>>(),
            expected,
            "{mode}: {findings:#?}"
        );
        for finding in findings {
            assert_eq!(
                finding.message.contains("leading SQL is unanalyzable"),
                source
                    .lines()
                    .nth(finding.line - 1)
                    .unwrap()
                    .contains("// unknown")
            );
        }
    }
}
