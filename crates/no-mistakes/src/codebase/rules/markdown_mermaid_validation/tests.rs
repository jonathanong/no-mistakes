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
    let files = [
        "invalid-state.md",
        "invalid-uppercase.MD",
        "invalid-mixed.MdX",
        "valid.md",
    ]
    .map(|name| root.join(name))
    .to_vec();

    let findings = check_with_files(&root, &config, &files).unwrap();

    assert_eq!(findings.len(), 3, "{findings:#?}");
    assert!(findings
        .iter()
        .any(|finding| finding.file == "invalid-state.md"));
    assert!(findings
        .iter()
        .any(|finding| finding.file == "invalid-uppercase.MD"));
    assert!(findings
        .iter()
        .any(|finding| finding.file == "invalid-mixed.MdX"));
}

#[test]
fn fact_preparation_uses_the_union_of_filtered_rule_targets() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/rules/markdown-mermaid-validation"),
    );
    let mut config =
        crate::config::v2::load_v2_config(&root, Some(&root.join(".no-mistakes.yml"))).unwrap();
    config.rules[0].include = vec!["invalid-uppercase.MD".to_string()];
    let mut second = config.rules[0].clone();
    second.include = vec!["*.mdx".to_string(), "*.MdX".to_string()];
    second.exclude = vec!["jsx-adjacent-invalid.mdx".to_string()];
    config.rules.push(second);
    let files = [
        "invalid-state.md",
        "invalid-uppercase.MD",
        "invalid-mixed.MdX",
        "jsx-adjacent-invalid.mdx",
        "valid.md",
    ]
    .map(|name| root.join(name));

    let targets = fact_candidate_files(&root, &config, &files).unwrap();
    assert_eq!(
        targets,
        vec![
            root.join("invalid-mixed.MdX"),
            root.join("invalid-uppercase.MD"),
        ]
    );

    let sources = super::super::source_store_for_files(&files);
    let mut plan = super::super::markdown_facts::MarkdownFactPlan::default();
    plan.request_pulldown(targets);
    let _facts = super::super::markdown_facts::MarkdownFactMap::prepare(&plan, &sources);
    assert_eq!(sources.physical_read_count(), 2);
}

#[test]
fn missing_facts_report_a_planning_error_and_partial_locations_render_defensively() {
    let facts = super::super::markdown_facts::MarkdownFactMap::default();
    let error = findings_for_path(
        Path::new("."),
        Path::new("missing.md"),
        &facts,
        &Analyzer::new(),
    )
    .unwrap_err();
    assert!(error
        .to_string()
        .contains("internal analysis-planning error"));

    let line_only = finding(diagnostic(Some(4), None));
    assert!(line_only.message.contains("at diagram line 4:"));
    let no_location = finding(diagnostic(None, Some(7)));
    assert_eq!(no_location.message, "invalid Mermaid diagram: syntax error");
}
