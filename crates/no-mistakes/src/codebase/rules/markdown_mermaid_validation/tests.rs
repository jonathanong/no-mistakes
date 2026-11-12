use super::*;

// Output cases moved to cli_mrf_snapshot_markdown_mermaid. Keep fact-candidate
// reads and partial diagnostic rendering below.

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

#[test]
fn standalone_mermaid_validation_matches_prepared_facts_for_saved_documents() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/rules/markdown-mermaid-validation"),
    );
    let config =
        crate::config::v2::load_v2_config(&root, Some(&root.join(".no-mistakes.yml"))).unwrap();
    let files = [
        "invalid-state.md",
        "invalid-uppercase.MD",
        "valid.md",
        "excluded.md",
    ]
    .map(|name| root.join(name));
    let standalone = check_with_files(&root, &config, &files).unwrap();
    assert_eq!(standalone.len(), 2);
    assert_eq!(
        standalone
            .iter()
            .map(|finding| finding.file.as_str())
            .collect::<Vec<_>>(),
        ["invalid-state.md", "invalid-uppercase.MD"]
    );
    assert!(standalone
        .iter()
        .all(|finding| finding.message.contains("invalid Mermaid diagram")));
    let sources = super::super::source_store_for_files(&files);
    let mut plan = super::super::markdown_facts::MarkdownFactPlan::default();
    plan.request_pulldown(fact_candidate_files(&root, &config, &files).unwrap());
    let facts = super::super::markdown_facts::MarkdownFactMap::prepare(&plan, &sources);
    let prepared = check_with_files_and_facts(&root, &config, &files, &facts).unwrap();
    assert_eq!(
        serde_json::to_value(standalone).unwrap(),
        serde_json::to_value(prepared).unwrap()
    );
    assert_eq!(sources.physical_read_count(), 3);
}

#[test]
fn standalone_mermaid_validation_surfaces_missing_source_instead_of_partial_results() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/rules/markdown-mermaid-validation"),
    );
    let config =
        crate::config::v2::load_v2_config(&root, Some(&root.join(".no-mistakes.yml"))).unwrap();
    let missing = root.join("not-present.md");
    assert!(!missing.exists());
    let error =
        check_with_files(&root, &config, &[root.join("invalid-state.md"), missing]).unwrap_err();
    assert!(format!("{error:#}").contains("not-present.md"), "{error:#}");
}
