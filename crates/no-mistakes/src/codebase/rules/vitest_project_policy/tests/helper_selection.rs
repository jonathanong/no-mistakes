use super::*;

#[test]
fn existing_literal_helpers_with_brackets_do_not_require_inventory_matches() {
    for case in [
        "helper-literal-file-brackets",
        "helper-literal-file-open-bracket",
        "helper-literal-dir-brackets",
        "helper-literal-dir-open-bracket",
    ] {
        let root = crate::test_support::rule_fixture_root("vitest-project-policy").join(case);
        let config =
            crate::config::v2::load_v2_config(&root, Some(&root.join(".no-mistakes.yml"))).unwrap();
        let files = vec![root.join("vitest.config.ts")];
        let sources = SourceStore::new(std::sync::Arc::new(
            crate::codebase::ts_source::FileInventory::from_paths(&files),
        ));
        let output =
            diagnostics_with_files_and_sources(&root, &config, &files, &sources, false).unwrap();
        assert!(
            output.incomplete.is_empty(),
            "{case}: {:?}",
            output.incomplete
        );
        assert_eq!(output.findings.len(), 2, "{case}: {:?}", output.findings);
        assert!(
            output
                .findings
                .iter()
                .any(|finding| finding.message.contains("must equal 0")),
            "{case}"
        );
        assert!(
            output
                .findings
                .iter()
                .any(|finding| finding.message.contains("declared.forbiddenValues")),
            "{case}"
        );
        assert!(
            output
                .findings
                .iter()
                .all(|finding| finding.file.starts_with("helpers/")),
            "{case}"
        );
    }
}
