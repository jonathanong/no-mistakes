use super::*;

#[test]
fn combined_markdown_demands_share_one_source_read() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/rules/markdown-mermaid-validation/valid.md");
    let sources = super::super::source_store_for_files(std::slice::from_ref(&path));
    let mut plan = MarkdownFactPlan::default();
    plan.request_pulldown([path.clone()]);
    plan.request_display_links([path.clone()]);

    let facts = MarkdownFactMap::prepare(&plan, &sources);

    assert_eq!(sources.physical_read_count(), 1);
    assert_eq!(
        facts
            .get_for_rule(&path, "test-rule")
            .unwrap()
            .mermaid_count,
        5
    );
}

#[test]
fn failed_markdown_reads_retain_the_source_store_failure() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/rules/markdown-read-failure/unreadable.md");
    let sources = super::super::source_store_for_files(std::slice::from_ref(&path));
    let mut plan = MarkdownFactPlan::default();
    plan.request_pulldown([path.clone()]);

    let facts = MarkdownFactMap::prepare(&plan, &sources);
    let stored = facts.by_path.get(&path).unwrap().as_ref().unwrap_err();
    let reread = sources.read_path(&path).unwrap_err();

    assert!(Arc::ptr_eq(stored, &reread));
    assert_eq!(sources.physical_read_count(), 1);
    let error = facts.get_for_rule(&path, "markdown-test").unwrap_err();
    assert!(error.to_string().contains("markdown-test could not read"));
    assert!(error.to_string().contains("readable UTF-8 file"));
}
