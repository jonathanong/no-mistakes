use super::*;

fn extract_all(source: &str) -> Vec<MermaidFence> {
    extract(
        source,
        0..source.len(),
        &mut MdxExpressionScanner::default(),
    )
    .fences
}

#[test]
fn recognizes_clear_mdx_jsx_without_treating_standard_html_as_mdx() {
    for source in [
        "<DiagramCard>\n```mermaid\nflowchart TD\n```\n</DiagramCard>\n",
        "<Card value={value}>\n```mermaid\nflowchart TD\n```\n</Card>\n",
        "<>\n```mermaid\nflowchart TD\n```\n</>\n",
    ] {
        assert!(looks_like_clear_mdx_jsx(source, 0..source.len()));
    }
    for source in [
        "plain text\n",
        "<div>\n```mermaid\nflowchart TD\n```\n</div>\n",
        "<DIV>\n```mermaid\nflowchart TD\n```\n</DIV>\n",
        "<div data-template=\"{name}\">\n```mermaid\nflowchart TD\n```\n</div>\n",
        "<div data-template='{name}'>\n```mermaid\nflowchart TD\n```\n</div>\n",
    ] {
        assert!(!looks_like_clear_mdx_jsx(source, 0..source.len()));
    }
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
fn extracts_and_normalizes_list_and_blockquote_containers() {
    let source = "<Card>\n- ```mermaid\n  flowchart TD\n  List --> Works\n  ```\n\n> ~~~mermaid\n> sequenceDiagram\n> A->>B: Works\n> ~~~\n</Card>\n";

    let fences = extract_all(source);

    assert_eq!(fences.len(), 2);
    assert_eq!(fences[0].content, "flowchart TD\nList --> Works\n");
    assert_eq!(fences[1].content, "sequenceDiagram\nA->>B: Works\n");
    assert!(fences.iter().all(|fence| fence.closed));
}

#[test]
fn container_closers_must_preserve_the_opening_context() {
    for source in [
        "- ```mermaid\n  flowchart TD\n```\n",
        "> ```mermaid\n> flowchart TD\n```\n",
        "> ```mermaid\n> flowchart TD\n\n> ```\n",
        "- ```mermaid\n  flowchart TD\n      ```\n",
    ] {
        let fences = extract_all(source);
        assert_eq!(fences.len(), 1, "{source:?}");
        assert!(!fences[0].closed, "{source:?}");
    }
}

#[test]
fn reports_unclosed_and_over_indented_closing_fences() {
    for source in [
        "```mermaid\nflowchart TD\nA-->B",
        "```mermaid\nflowchart TD\nA-->B\n    ```\n",
        "```mermaid\nflowchart TD\nA-->B\n```\x0c\n",
        "~~~mermaid\nflowchart TD\nA-->B\n~~~\x0b\n",
    ] {
        let fences = extract_all(source);
        assert_eq!(fences.len(), 1);
        assert!(!fences[0].closed);
    }
}

#[test]
fn recognizes_only_commonmark_atx_heading_shapes() {
    for heading in [b"# heading".as_slice(), b"   ######\theading", b"##"] {
        assert!(is_atx_heading(heading), "{heading:?}");
    }
    for paragraph in [
        b"####### heading".as_slice(),
        b"#not-a-heading",
        b"    # indented code",
    ] {
        assert!(!is_atx_heading(paragraph), "{paragraph:?}");
    }
}

#[test]
fn recognizes_only_commonmark_thematic_break_shapes() {
    assert!(!is_thematic_break(b""));
    for thematic_break in [
        b"***".as_slice(),
        b" * * * ",
        b"  ---",
        b"   _ _ _ _   ",
        b"*\t*\t*\t",
    ] {
        assert!(is_thematic_break(thematic_break), "{thematic_break:?}");
    }
    for paragraph in [
        b"**".as_slice(),
        b"*-*",
        b"--- content",
        b"- - _",
        b"    ***",
    ] {
        assert!(!is_thematic_break(paragraph), "{paragraph:?}");
    }
}

#[test]
fn recognizes_setext_and_container_block_boundaries() {
    assert!(!is_setext_heading_underline(b""));
    for underline in [b"=".as_slice(), b" ===  ", b"---\t"] {
        assert!(is_setext_heading_underline(underline), "{underline:?}");
    }
    for prose in [b"= =".as_slice(), b"=-", b"=== title", b"    ==="] {
        assert!(!is_setext_heading_underline(prose), "{prose:?}");
    }

    for container in [
        b">quote".as_slice(),
        b"  > quote",
        b"- item",
        b" * item",
        b"1. item",
        b"01) item",
    ] {
        assert!(starts_block_container(container, true), "{container:?}");
    }
    assert!(!starts_block_container(b"", false));
    for prose in [
        b"    > quote".as_slice(),
        b"-",
        b"-   ",
        b"*not-a-list",
        b"2. item",
        b"1.item",
        b"1.   ",
        b"0000000001. item",
    ] {
        assert!(!starts_block_container(prose, true), "{prose:?}");
    }

    for container in [b"2. item".as_slice(), b"3)", b"-", b"-   "] {
        assert!(starts_block_container(container, false), "{container:?}");
    }
    for paragraph in [b"2. item".as_slice(), b"1.", b"-", b"-   "] {
        assert!(!starts_block_container(paragraph, true), "{paragraph:?}");
    }

    for code in [b"    code".as_slice(), b"\tcode", b" \tcode"] {
        assert!(is_indented_code(code), "{code:?}");
    }
    assert!(!is_indented_code(b"   code"));
}
