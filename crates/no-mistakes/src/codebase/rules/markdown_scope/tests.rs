use super::*;

#[test]
fn mermaid_extensions_are_ascii_case_insensitive() {
    let files = [
        PathBuf::from("README.MD"),
        PathBuf::from("guide.MarkDown"),
        PathBuf::from("component.MdX"),
        PathBuf::from("source.ts"),
    ];

    assert_eq!(
        mermaid_document_files(&files),
        vec![
            PathBuf::from("README.MD"),
            PathBuf::from("component.MdX"),
            PathBuf::from("guide.MarkDown"),
        ]
    );
    assert!(markdown_files(&files).is_empty());
    assert!(is_mermaid_document(Path::new("README.MD")));
    assert!(!is_mermaid_document(Path::new("source.ts")));
}
