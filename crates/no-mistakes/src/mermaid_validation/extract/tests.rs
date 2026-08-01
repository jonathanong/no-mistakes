use super::*;

#[test]
fn delimiter_and_line_helpers_cover_non_fence_and_crlf_edges() {
    assert!(opening_delimiter("plain text", usize::MAX).is_none());
    assert!(opening_delimiter("`` mermaid", 0).is_none());
    assert!(opening_delimiter("```text\n", 0).is_none());
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
    let delimiter = opening_delimiter("-\t~~~mermaid\n", 0).expect("Mermaid fence delimiter");

    assert_eq!(delimiter.container_indent, 4);
}
