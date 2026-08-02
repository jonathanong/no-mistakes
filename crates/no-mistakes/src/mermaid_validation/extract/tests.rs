use super::*;

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
