use super::*;
use std::path::PathBuf;

fn fixture(name: &str) -> String {
    std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/rules/markdown-mermaid-validation")
            .join(name),
    )
    .expect("Mermaid Markdown fixture should be readable")
}

#[test]
fn delimiter_and_line_helpers_cover_non_fence_and_crlf_edges() {
    assert!(opening_delimiter("plain text", usize::MAX, false).is_none());
    assert!(opening_delimiter("`` mermaid", 0, false).is_none());
    assert!(opening_delimiter("```text\n", 0, false).is_none());
    assert_eq!(line_number("one\r\ntwo\rthree", usize::MAX), 3);

    let source = "plain text";
    let mut collector = MermaidFenceCollector::new(source);
    collector.observe(
        &Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced("mermaid".into()))),
        0..0,
    );
    assert!(collector.finish().is_empty());
}

#[test]
fn opening_delimiter_measures_tabbed_list_prefix_in_markdown_columns() {
    let delimiter = opening_delimiter("-\t~~~mermaid\n", 0, true).expect("Mermaid fence delimiter");

    assert_eq!(delimiter.container_indent, 4);
}

#[test]
fn opening_delimiter_retains_whitespace_indent_for_list_continuations() {
    let source = "  ```mermaid\n";

    assert_eq!(
        opening_delimiter(source, 0, true)
            .expect("list Mermaid fence delimiter")
            .container_indent,
        2
    );
    assert_eq!(
        opening_delimiter(source, 0, false)
            .expect("top-level Mermaid fence delimiter")
            .container_indent,
        0
    );
}

#[test]
fn overlapping_mdx_code_block_reuses_already_scanned_source() {
    let source = fixture("mdx-expression-fence-text.mdx");
    let mermaid_at = source
        .find("```mermaid")
        .expect("saved fixture keeps a real Mermaid fence");
    let mut collector = MermaidFenceCollector::new_with_mdx_html_fallback(&source);
    collector.observe(&Event::Start(Tag::HtmlBlock), 0..mermaid_at);
    assert!(
        collector.recover_overlapping_mdx_code_block(mermaid_at - 1..source.len()),
        "a later code-block range must overlap the already-scanned MDX prefix"
    );
    assert!(
        !collector.recover_overlapping_mdx_code_block(source.len()..source.len()),
        "a range at the scanned end is not overlapping"
    );
}

#[cfg(feature = "mermaid-validation")]
#[test]
fn overlapping_mdx_document_reports_the_saved_fence_once() {
    // The overlap predicate only returns a boolean. This parsed document must
    // still report the real fence once and ignore the tilde fence in the
    // JavaScript template. The wrapper exists only with mermaid-validation,
    // and the no-default-features lib tests still compile this module.
    let source = fixture("mdx-expression-fence-text.mdx");
    let mermaid_at = source
        .find("```mermaid")
        .expect("saved fixture keeps a real Mermaid fence");
    let fences = super::extract_mermaid_fences_with_mdx_html_fallback(&source);
    assert_eq!(
        fences.len(),
        1,
        "the real Mermaid fence is reported once: {fences:?}"
    );
    assert_eq!(fences[0].fence_offset, mermaid_at);
    assert_eq!(
        fences[0].fence_line,
        super::line_number(&source, mermaid_at)
    );
    assert!(fences[0].closed, "{fences:?}");
    assert!(
        fences[0].content.contains("Parsed --> Diagram"),
        "illustrative tilde fence must not replace the real diagram: {}",
        fences[0].content
    );
}

#[test]
fn overlapping_recovery_records_the_fence_it_scans() {
    // The saved document starts with a newline, so the scanned prefix is only
    // that newline and recovery's extract starts on the fence. Dropping the
    // extracted fences leaves this empty.
    let source = fixture("overlapping-recovery-fence.mdx");
    let fence_at = source
        .find("```mermaid")
        .expect("fence starts after the scanned newline");
    let mut collector = MermaidFenceCollector::new_with_mdx_html_fallback(&source);
    collector.observe(&Event::Start(Tag::HtmlBlock), 0..fence_at);
    assert!(
        collector.recover_overlapping_mdx_code_block(0..source.len()),
        "the code block starts inside the already scanned prefix"
    );
    let fences = collector.finish();
    assert_eq!(fences.len(), 1, "{fences:?}");
    assert_eq!(fences[0].fence_offset, fence_at);
    assert_eq!(fences[0].fence_line, super::line_number(&source, fence_at));
    assert!(fences[0].closed, "{fences:?}");
    assert!(
        fences[0].content.contains("Parsed --> Diagram"),
        "{}",
        fences[0].content
    );
}
