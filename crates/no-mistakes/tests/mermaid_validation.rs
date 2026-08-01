use no_mistakes::mermaid_validation::{validate_markdown, MermaidValidationDiagnosticCode};
use std::path::PathBuf;

fn fixture(name: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/rules/markdown-mermaid-validation")
        .join(name);
    std::fs::read_to_string(path).expect("Mermaid Markdown fixture should be readable")
}

#[test]
fn validates_supported_commonmark_fences() {
    let result = validate_markdown(&fixture("valid.md"), Some("docs/diagrams.md"));

    assert!(result.valid, "{:#?}", result.diagnostics);
    assert_eq!(result.diagram_count, 5);
    assert!(result.diagnostics.is_empty());
}

#[test]
fn reports_only_the_invalid_diagram_in_a_multi_diagram_document() {
    let result = validate_markdown(&fixture("multiple.md"), Some("docs/multiple.md"));

    assert!(!result.valid);
    assert_eq!(result.diagram_count, 2);
    assert_eq!(result.diagnostics.len(), 1);
    let diagnostic = &result.diagnostics[0];
    assert_eq!(
        diagnostic.code,
        MermaidValidationDiagnosticCode::InvalidSyntax
    );
    assert_eq!(diagnostic.file, "docs/multiple.md");
    assert_eq!(diagnostic.fence_line, 8);
    assert!(diagnostic.diagram_line.is_some());
    assert!(diagnostic.diagram_column.is_some());
    assert!(diagnostic.diagram_type.is_some());
    assert!(!diagnostic.message.is_empty());
}

#[test]
fn reports_an_unclosed_fence_without_a_cascading_syntax_error() {
    let result = validate_markdown(&fixture("unclosed.md"), None);

    assert!(!result.valid);
    assert_eq!(result.diagram_count, 1);
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(
        result.diagnostics[0].code,
        MermaidValidationDiagnosticCode::UnclosedFence
    );
    assert_eq!(result.diagnostics[0].file, "<input>");
    assert_eq!(result.diagnostics[0].fence_line, 3);
    assert_eq!(result.diagnostics[0].diagram_line, None);
    assert_eq!(result.diagnostics[0].diagram_column, None);
    assert_eq!(result.diagnostics[0].diagram_type, None);
}

#[test]
fn ignores_non_mermaid_code_fences() {
    let result = validate_markdown(&fixture("ignored.md"), None);

    assert!(result.valid);
    assert_eq!(result.diagram_count, 0);
    assert!(result.diagnostics.is_empty());
}
