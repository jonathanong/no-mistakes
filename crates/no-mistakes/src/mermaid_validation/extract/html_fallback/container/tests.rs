use super::*;

#[test]
fn strips_nested_blockquote_and_list_containers() {
    let (opening, prefix) = ContainerPrefix::from_opening_line(b"> > - - ```mermaid");

    assert_eq!(opening, b"```mermaid");
    assert_eq!(
        prefix.strip_line(b"> >     flowchart TD"),
        Some(&b"flowchart TD"[..])
    );
    assert_eq!(prefix.strip_line(b"> >     ```"), Some(&b"```"[..]));
    assert_eq!(prefix.strip_line(b"  ```"), None);
}

#[test]
fn recognizes_ordered_markers_and_rejects_lookalikes() {
    let (opening, ordered) = ContainerPrefix::from_opening_line(b"123. ```mermaid");
    assert_eq!(opening, b"```mermaid");
    assert_eq!(ordered.strip_line(b"     graph TD"), Some(&b"graph TD"[..]));

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
    assert_eq!(padded.strip_line(b"  graph TD"), Some(&b"graph TD"[..]));

    let (opening, tabbed) = ContainerPrefix::from_opening_line(b"-\t```mermaid");
    assert_eq!(opening, b"```mermaid");
    assert_eq!(tabbed.strip_line(b"\tgraph TD"), Some(&b"graph TD"[..]));
}

#[test]
fn preserves_non_container_lines_and_normalizes_blank_lines() {
    let (opening, direct) = ContainerPrefix::from_opening_line(b" ```mermaid");
    assert_eq!(opening, b" ```mermaid");
    assert_eq!(direct.strip_line(b" graph TD"), Some(&b" graph TD"[..]));

    let (_, quoted) = ContainerPrefix::from_opening_line(b"> ```mermaid");
    assert_eq!(quoted.strip_line(b"   "), Some(&b""[..]));
    assert_eq!(quoted.strip_line(b"    > graph TD"), None);
    assert_eq!(strip_indentation(b"", 2), None);
}
