use super::*;

fn extract_all(source: &str) -> Vec<MermaidFence> {
    extract(source, 0..source.len())
}

#[test]
fn ignores_non_fences_and_invalid_opening_delimiters() {
    for source in [
        "plain text\n",
        "    ```mermaid\nflowchart TD\n```\n",
        "\t```mermaid\nflowchart TD\n```\n",
        "``mermaid\nflowchart TD\n``\n",
        "```mermaid `title`\nflowchart TD\n```\n",
    ] {
        assert!(
            extract_all(source).is_empty(),
            "unexpected fence: {source:?}"
        );
    }
}

#[test]
fn skips_non_mermaid_fences_before_extracting_mermaid() {
    let source = "```text\n```mermaid\n```\n```mermaid\nflowchart TD\nA-->B\n```\n";

    let fences = extract_all(source);

    assert_eq!(fences.len(), 1);
    assert!(fences[0].closed);
    assert!(fences[0].content.contains("flowchart TD"));
    assert_eq!(fences[0].fence_line, 4);
}

#[test]
fn extracts_tilde_fences_with_crlf_line_endings() {
    let source = "~~~ Mermaid title=Sequence\r\nsequenceDiagram\r\nA->>B: Hi\r\n~~~~\r\n";

    let fences = extract_all(source);

    assert_eq!(fences.len(), 1);
    assert!(fences[0].closed);
    assert_eq!(fences[0].fence_offset, 0);
    assert!(fences[0].content.contains("sequenceDiagram\r\n"));
}

#[test]
fn reports_unclosed_and_over_indented_closing_fences() {
    for source in [
        "```mermaid\nflowchart TD\nA-->B",
        "```mermaid\nflowchart TD\nA-->B\n    ```\n",
    ] {
        let fences = extract_all(source);
        assert_eq!(fences.len(), 1);
        assert!(!fences[0].closed);
    }
}
