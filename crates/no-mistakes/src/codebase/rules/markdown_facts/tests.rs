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
    assert_eq!(facts.get(&path).unwrap().mermaid_count, 5);
}
