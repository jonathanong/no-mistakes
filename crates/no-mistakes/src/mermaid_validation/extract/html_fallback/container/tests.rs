use super::*;

#[test]
fn strips_nested_blockquote_and_list_containers() {
    let (opening, prefix) = ContainerPrefix::from_opening_line(b"> > - - ```mermaid");

    assert_eq!(opening, b"```mermaid");
    assert_eq!(
        prefix.strip_line(b"> >     flowchart TD").as_deref(),
        Some(&b"flowchart TD"[..])
    );
    assert_eq!(
        prefix.strip_line(b"> >     ```").as_deref(),
        Some(&b"```"[..])
    );
    assert_eq!(prefix.strip_line(b"  ```"), None);
}

#[test]
fn strips_interleaved_container_steps_in_opening_order() {
    let (opening, prefix) = ContainerPrefix::from_opening_line(b"> - > 1. ~~~mermaid");

    assert_eq!(opening, b"~~~mermaid");
    assert_eq!(
        prefix.strip_line(b">   >    flowchart TD").as_deref(),
        Some(&b"flowchart TD"[..])
    );
    assert_eq!(
        prefix.strip_line(b">   >    ~~~").as_deref(),
        Some(&b"~~~"[..])
    );
    assert_eq!(prefix.strip_line(b"> >      ~~~"), None);
    assert_eq!(prefix.strip_line(b"  >    ~~~"), None);
}

#[test]
fn recognizes_ordered_markers_and_rejects_lookalikes() {
    let (opening, ordered) = ContainerPrefix::from_opening_line(b"123. ```mermaid");
    assert_eq!(opening, b"```mermaid");
    assert!(!ordered.can_interrupt_paragraph());
    assert_eq!(
        ordered.strip_line(b"     graph TD").as_deref(),
        Some(&b"graph TD"[..])
    );

    let (_, one) = ContainerPrefix::from_opening_line(b"01. ```mermaid");
    assert!(one.can_interrupt_paragraph());

    for source in [
        b"1234567890. ```mermaid".as_slice(),
        b"12x ```mermaid",
        b"-not-a-list",
        b"    - ```mermaid",
    ] {
        let (opening, _) = ContainerPrefix::from_opening_line(source);
        assert_eq!(opening, source);
    }

    let (opening, padded) = ContainerPrefix::from_opening_line(b"-     ```mermaid");
    assert_eq!(opening, b"    ```mermaid");
    assert_eq!(
        padded.strip_line(b"  graph TD").as_deref(),
        Some(&b"graph TD"[..])
    );

    let (opening, tabbed) = ContainerPrefix::from_opening_line(b"-\t```mermaid");
    assert_eq!(opening, b"```mermaid");
    assert_eq!(
        tabbed.strip_line(b"\tgraph TD").as_deref(),
        Some(&b"graph TD"[..])
    );

    let (opening, overwide_tabbed_padding) =
        ContainerPrefix::from_opening_line(b"-   \t```mermaid");
    assert_eq!(opening, b"  \t```mermaid");
    assert_eq!(overwide_tabbed_padding.steps.len(), 1);
}

#[test]
fn preserves_non_container_lines_and_normalizes_blank_lines() {
    let (opening, direct) = ContainerPrefix::from_opening_line(b" ```mermaid");
    assert_eq!(opening, b" ```mermaid");
    assert_eq!(
        direct.strip_line(b" graph TD").as_deref(),
        Some(&b" graph TD"[..])
    );
    assert_eq!(direct.strip_line(b" \t").as_deref(), Some(&b""[..]));

    let (_, quoted) = ContainerPrefix::from_opening_line(b"> ```mermaid");
    assert_eq!(
        quoted.strip_line(b">\troot").as_deref(),
        Some(&b"  root"[..])
    );
    assert_eq!(
        quoted.strip_line(b"   >\troot").as_deref(),
        Some(&b"   root"[..])
    );
    assert_eq!(quoted.strip_line(b">").as_deref(), Some(&b""[..]));
    assert_eq!(quoted.strip_line(b"   "), None);
    assert_eq!(quoted.strip_line(b" \t"), None);
    assert_eq!(quoted.strip_line(b"\x0c"), None);
    assert_eq!(quoted.strip_line(b"\x0b"), None);
    assert_eq!(quoted.strip_line(b"    > graph TD"), None);
    assert_eq!(indentation_prefix(b"", 2), None);
}

#[test]
fn preserves_spaces_when_a_tab_overshoots_list_indentation() {
    let (_, prefix) = ContainerPrefix::from_opening_line(b"- ```mermaid");
    assert_eq!(
        prefix.strip_line(b"\tgraph TD").as_deref(),
        Some(&b"  graph TD"[..])
    );

    // Exercise a later container step against the owned residual-space buffer.
    let (_, interleaved) = ContainerPrefix::from_opening_line(b"- > ```mermaid");
    assert_eq!(
        interleaved.strip_line(b"\t> graph TD").as_deref(),
        Some(&b"graph TD"[..])
    );
}
