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
fn tab_indented_fence_does_not_close_a_top_level_block() {
    let result = validate_markdown(
        &fixture("unclosed-tab-indented.md"),
        Some("docs/tab-indented.md"),
    );

    assert!(!result.valid);
    assert_eq!(result.diagram_count, 1);
    assert_eq!(result.diagnostics.len(), 1);
    let diagnostic = &result.diagnostics[0];
    assert_eq!(
        diagnostic.code,
        MermaidValidationDiagnosticCode::UnclosedFence
    );
    assert_eq!(diagnostic.file, "docs/tab-indented.md");
    assert_eq!(diagnostic.fence_line, 3);
    assert_eq!(diagnostic.diagram_line, None);
    assert_eq!(diagnostic.diagram_column, None);
    assert_eq!(diagnostic.diagram_type, None);
}

#[test]
fn closing_fences_require_the_opening_blockquote_depth() {
    for fixture_name in [
        "unclosed-top-level-quoted-closer.md",
        "unclosed-blockquote-wrong-depth.md",
    ] {
        let result = validate_markdown(&fixture(fixture_name), Some(fixture_name));

        assert!(!result.valid, "{fixture_name}");
        assert_eq!(result.diagram_count, 1, "{fixture_name}");
        assert_eq!(result.diagnostics.len(), 1, "{fixture_name}");
        assert_eq!(
            result.diagnostics[0].code,
            MermaidValidationDiagnosticCode::UnclosedFence,
            "{fixture_name}"
        );
        assert_eq!(result.diagnostics[0].fence_line, 3, "{fixture_name}");
        assert_eq!(result.diagnostics[0].diagram_line, None, "{fixture_name}");
    }
}

#[test]
fn validates_a_fence_directly_inside_mdx_jsx() {
    let result = validate_markdown(
        &fixture("jsx-adjacent-invalid.mdx"),
        Some("docs/component.mdx"),
    );

    assert!(!result.valid);
    assert_eq!(result.diagram_count, 1);
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(
        result.diagnostics[0].code,
        MermaidValidationDiagnosticCode::InvalidSyntax
    );
    assert_eq!(result.diagnostics[0].file, "docs/component.mdx");
    assert_eq!(result.diagnostics[0].fence_line, 4);
}

#[test]
fn omitted_file_auto_detects_a_fence_inside_clear_mdx_jsx() {
    let result = validate_markdown(&fixture("jsx-adjacent-invalid.mdx"), None);

    assert!(!result.valid);
    assert_eq!(result.diagram_count, 1);
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(
        result.diagnostics[0].code,
        MermaidValidationDiagnosticCode::InvalidSyntax
    );
    assert_eq!(result.diagnostics[0].file, "<input>");
    assert_eq!(result.diagnostics[0].fence_line, 4);
}

#[test]
fn omitted_file_preserves_standard_markdown_html_block_semantics() {
    let result = validate_markdown(&fixture("standard-html-block.md"), None);

    assert!(result.valid);
    assert_eq!(result.diagram_count, 0);
    assert!(result.diagnostics.is_empty());
}

#[test]
fn validates_list_and_blockquote_fences_inside_mdx_jsx() {
    let result = validate_markdown(
        &fixture("jsx-containers-valid.mdx"),
        Some("docs/containers.mdx"),
    );

    assert!(result.valid, "{:#?}", result.diagnostics);
    assert_eq!(result.diagram_count, 2);
    assert!(result.diagnostics.is_empty());
}

#[test]
fn markdown_file_does_not_enable_mdx_jsx_recovery() {
    let result = validate_markdown(
        &fixture("jsx-adjacent-invalid.mdx"),
        Some("docs/component.md"),
    );

    assert!(result.valid);
    assert_eq!(result.diagram_count, 0);
    assert!(result.diagnostics.is_empty());
}

#[test]
fn ignores_non_mermaid_code_fences() {
    let result = validate_markdown(&fixture("ignored.md"), None);

    assert!(result.valid);
    assert_eq!(result.diagram_count, 0);
    assert!(result.diagnostics.is_empty());
}
