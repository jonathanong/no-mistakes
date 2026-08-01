use super::*;

fn diagnostic(
    diagram_line: Option<usize>,
    diagram_column: Option<usize>,
) -> MermaidValidationDiagnostic {
    MermaidValidationDiagnostic {
        code: MermaidValidationDiagnosticCode::InvalidSyntax,
        file: "diagram.md".to_string(),
        fence_line: 2,
        diagram_line,
        diagram_column,
        diagram_type: None,
        message: "syntax error".to_string(),
    }
}

#[test]
fn standalone_rule_wrapper_reads_saved_markdown_fixtures() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/rules/markdown-mermaid-validation");
    let config =
        crate::config::v2::load_v2_config(&root, Some(&root.join(".no-mistakes.yml"))).unwrap();
    let files = ["invalid-state.md", "valid.md"]
        .map(|name| root.join(name))
        .to_vec();

    let findings = check_with_files(&root, &config, &files).unwrap();

    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].file, "invalid-state.md");
}

#[test]
fn missing_facts_and_partial_locations_are_rendered_defensively() {
    let facts = super::super::markdown_facts::MarkdownFactMap::default();
    assert!(findings_for_path(Path::new("."), Path::new("missing.md"), &facts).is_empty());

    let line_only = finding(diagnostic(Some(4), None));
    assert!(line_only.message.contains("at diagram line 4:"));
    let no_location = finding(diagnostic(None, Some(7)));
    assert_eq!(no_location.message, "invalid Mermaid diagram: syntax error");
}
