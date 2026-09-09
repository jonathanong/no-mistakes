#[test]
fn file_traversal_scans_only_selected_files() {
    let selection = include_str!("../selection.rs");
    assert!(
        selection.contains("graph.call_sites_in_file(&file)"),
        "file traversal must iterate selected files' call sites"
    );
    assert!(
        selection.contains("file traversal returns the expanded roots"),
        "file traversal must not BFS the unselected universe"
    );
}
