use no_mistakes::mermaid_validation::{validate_markdown, MermaidValidationDiagnosticCode};
use std::path::PathBuf;

fn fixture(name: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/rules/markdown-mermaid-validation")
        .join(name);
    std::fs::read_to_string(path)
        .expect("Mermaid Markdown fixture should be readable")
        // Keep control-character regressions representable in Git's text-only fixtures.
        .replace("{{FORM_FEED}}", "\u{000c}")
        .replace("{{VERTICAL_TAB}}", "\u{000b}")
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
fn validates_an_mdx_jsx_fence_split_across_html_block_ranges() {
    for file in [Some("docs/blank-line.mdx"), None] {
        let result = validate_markdown(&fixture("jsx-blank-line-valid.mdx"), file);

        assert!(result.valid, "{file:?}: {:#?}", result.diagnostics);
        assert_eq!(result.diagram_count, 1, "{file:?}");
        assert!(result.diagnostics.is_empty(), "{file:?}");
    }
}

#[test]
fn resumes_after_a_non_mermaid_mdx_fence_split_across_html_ranges() {
    let result = validate_markdown(
        &fixture("jsx-non-mermaid-blank-line.mdx"),
        Some("docs/non-mermaid-first.mdx"),
    );

    assert!(!result.valid);
    assert_eq!(result.diagram_count, 1);
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(result.diagnostics[0].fence_line, 8);
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
fn omitted_file_ignores_braces_inside_quoted_standard_html_attributes() {
    let result = validate_markdown(&fixture("standard-html-quoted-braces.md"), None);

    assert!(result.valid);
    assert_eq!(result.diagram_count, 0);
    assert!(result.diagnostics.is_empty());
}

#[test]
fn tabbed_list_fence_closer_uses_markdown_columns() {
    let result = validate_markdown(
        &fixture("tabbed-list-valid-closer.md"),
        Some("docs/tabbed-list.md"),
    );

    assert!(result.valid, "{:#?}", result.diagnostics);
    assert_eq!(result.diagram_count, 1);
    assert!(result.diagnostics.is_empty());
}

#[test]
fn list_continuation_closer_allows_container_indent_plus_three_columns() {
    let result = validate_markdown(
        &fixture("list-continuation-valid-closer.md"),
        Some("docs/list-continuation.md"),
    );

    assert!(result.valid, "{:#?}", result.diagnostics);
    assert_eq!(result.diagram_count, 1);
    assert!(result.diagnostics.is_empty());
}

#[test]
fn blockquote_tab_padding_uses_the_original_markdown_column() {
    let result = validate_markdown(
        &fixture("blockquote-tab-padding-valid.md"),
        Some("docs/blockquote-tab.md"),
    );

    assert!(result.valid, "{:#?}", result.diagnostics);
    assert_eq!(result.diagram_count, 1);
    assert!(result.diagnostics.is_empty());
}

#[test]
fn closing_fence_suffix_allows_only_spaces_and_tabs() {
    for fixture_name in [
        "unclosed-form-feed-suffix.md",
        "unclosed-mdx-vertical-tab-suffix.mdx",
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
    }
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
fn non_one_ordered_list_marker_does_not_interrupt_an_mdx_paragraph() {
    let result = validate_markdown(
        &fixture("jsx-ordered-list-interruption.mdx"),
        Some("docs/list-interruption.mdx"),
    );

    assert!(result.valid, "{:#?}", result.diagnostics);
    assert_eq!(result.diagram_count, 0);
    assert!(result.diagnostics.is_empty());
}

#[test]
fn non_one_ordered_list_marker_can_follow_an_mdx_atx_heading() {
    let result = validate_markdown(
        &fixture("jsx-heading-ordered-list.mdx"),
        Some("docs/heading-list.mdx"),
    );

    assert!(!result.valid);
    assert_eq!(result.diagram_count, 1);
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(result.diagnostics[0].fence_line, 3);
}

#[test]
fn non_one_ordered_list_marker_can_follow_an_mdx_thematic_break() {
    let result = validate_markdown(
        &fixture("jsx-thematic-break-ordered-list.mdx"),
        Some("docs/thematic-list.mdx"),
    );

    assert!(!result.valid);
    assert_eq!(result.diagram_count, 1);
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(result.diagnostics[0].fence_line, 4);
}

#[test]
fn non_one_ordered_list_markers_can_follow_other_mdx_block_boundaries() {
    let result = validate_markdown(
        &fixture("jsx-block-boundary-ordered-lists.mdx"),
        Some("docs/block-boundaries.mdx"),
    );

    assert!(!result.valid);
    assert_eq!(result.diagram_count, 6);
    assert_eq!(result.diagnostics.len(), 6);
}

#[test]
fn non_one_ordered_list_marker_can_follow_an_mdx_flow_jsx_boundary() {
    let result = validate_markdown(
        &fixture("jsx-flow-boundary-ordered-list.mdx"),
        Some("docs/flow-boundary.mdx"),
    );

    assert!(!result.valid);
    assert_eq!(result.diagram_count, 2);
    assert_eq!(result.diagnostics.len(), 2);
    assert_eq!(
        result
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.fence_line)
            .collect::<Vec<_>>(),
        vec![4, 13]
    );
}

#[test]
fn validates_mdx_list_lines_with_tab_overshoot() {
    let result = validate_markdown(
        &fixture("mdx-tab-overshoot-valid.mdx"),
        Some("docs/tab-overshoot.mdx"),
    );

    assert!(result.valid, "{:#?}", result.diagnostics);
    assert_eq!(result.diagram_count, 1);
    assert!(result.diagnostics.is_empty());
}

#[test]
fn preserves_mdx_blockquote_tab_residual_indentation() {
    let result = validate_markdown(
        &fixture("mdx-blockquote-tab-residual-valid.mdx"),
        Some("docs/blockquote-tab-residual.mdx"),
    );

    assert!(result.valid, "{:#?}", result.diagnostics);
    assert_eq!(result.diagram_count, 1);
    assert!(result.diagnostics.is_empty());
}

#[test]
fn ignores_mdx_fence_text_after_overwide_list_padding() {
    let result = validate_markdown(
        &fixture("mdx-overwide-list-padding-ignored.mdx"),
        Some("docs/overwide-list-padding.mdx"),
    );

    assert!(result.valid, "{:#?}", result.diagnostics);
    assert_eq!(result.diagram_count, 0);
    assert!(result.diagnostics.is_empty());
}

#[test]
fn container_blank_lines_allow_only_spaces_and_tabs() {
    for fixture_name in [
        "unclosed-mdx-form-feed-container-blank.mdx",
        "unclosed-mdx-vertical-tab-container-blank.mdx",
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
    }
}

#[test]
fn unmarked_blank_line_ends_an_mdx_blockquote_fence() {
    let result = validate_markdown(
        &fixture("unclosed-mdx-blockquote-unmarked-blank.mdx"),
        Some("docs/blockquote-unmarked-blank.mdx"),
    );

    assert!(!result.valid);
    assert_eq!(result.diagram_count, 1);
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(
        result.diagnostics[0].code,
        MermaidValidationDiagnosticCode::UnclosedFence
    );
    assert_eq!(result.diagnostics[0].fence_line, 4);
}

#[test]
fn validates_interleaved_container_fences_inside_mdx_jsx() {
    let result = validate_markdown(
        &fixture("jsx-interleaved-containers-valid.mdx"),
        Some("docs/interleaved-containers.mdx"),
    );

    assert!(result.valid, "{:#?}", result.diagnostics);
    assert_eq!(result.diagram_count, 2);
    assert!(result.diagnostics.is_empty());
}

#[test]
fn ignores_fence_like_text_inside_an_mdx_javascript_expression() {
    for file in [Some("docs/expression.mdx"), None] {
        let result = validate_markdown(&fixture("mdx-expression-fence-text.mdx"), file);

        assert!(result.valid, "{file:?}: {:#?}", result.diagnostics);
        assert_eq!(result.diagram_count, 1, "{file:?}");
        assert!(result.diagnostics.is_empty(), "{file:?}");
    }
}

#[test]
fn ignores_fence_text_in_mdx_container_paragraphs() {
    let result = validate_markdown(
        &fixture("mdx-container-paragraph-fence-text.mdx"),
        Some("docs/container-paragraph.mdx"),
    );

    assert!(result.valid, "{:#?}", result.diagnostics);
    assert_eq!(result.diagram_count, 0);
    assert!(result.diagnostics.is_empty());
}

#[test]
fn ignores_fence_text_in_multiline_commonmark_code_spans() {
    let result = validate_markdown(
        &fixture("mdx-multiline-code-span-fence-text.mdx"),
        Some("docs/code-span.mdx"),
    );

    assert!(result.valid, "{:#?}", result.diagnostics);
    assert_eq!(result.diagram_count, 0);
    assert!(result.diagnostics.is_empty());
}

#[test]
fn ignores_fence_like_text_inside_top_level_mdx_code_regions() {
    let file = Some("docs/top-level-code.mdx");
    let result = validate_markdown(&fixture("mdx-top-level-code-regions.mdx"), file);

    assert!(result.valid, "{file:?}: {:#?}", result.diagnostics);
    assert_eq!(result.diagram_count, 1, "{file:?}");
    assert!(result.diagnostics.is_empty(), "{file:?}");
}

#[test]
fn ignores_fence_like_text_after_an_esm_leading_continuation() {
    let result = validate_markdown(
        &fixture("mdx-esm-leading-continuation.mdx"),
        Some("docs/esm-continuation.mdx"),
    );

    assert!(result.valid, "{:#?}", result.diagnostics);
    assert_eq!(result.diagram_count, 1);
    assert!(result.diagnostics.is_empty());
}

#[test]
fn handles_mdx_expression_boundaries_and_esm_comments() {
    for fixture_name in [
        "mdx-escaped-expression-brace.mdx",
        "mdx-same-line-expressions.mdx",
        "mdx-esm-trailing-comments.mdx",
    ] {
        let file = Some(fixture_name);
        let result = validate_markdown(&fixture(fixture_name), file);

        assert!(
            result.valid,
            "{fixture_name} ({file:?}): {:#?}",
            result.diagnostics
        );
        assert_eq!(result.diagram_count, 1, "{fixture_name} ({file:?})");
    }
}

#[test]
fn automatic_mode_preserves_commonmark_until_clear_mdx_is_detected() {
    let result = validate_markdown(&fixture("automatic-commonmark-unmatched-brace.md"), None);

    assert!(result.valid, "{:#?}", result.diagnostics);
    assert_eq!(result.diagram_count, 1);

    let mdx = validate_markdown(&fixture("mdx-expression-fence-text.mdx"), None);
    assert!(mdx.valid, "{:#?}", mdx.diagnostics);
    assert_eq!(mdx.diagram_count, 1);
}

#[test]
fn finds_mermaid_after_nested_jsx_with_apostrophes_in_text() {
    for file in [Some("docs/nested-jsx.mdx"), None] {
        let result = validate_markdown(&fixture("mdx-expression-nested-jsx.mdx"), file);

        assert!(result.valid, "{file:?}: {:#?}", result.diagnostics);
        assert_eq!(result.diagram_count, 1, "{file:?}");
    }
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
