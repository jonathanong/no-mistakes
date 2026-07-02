use super::*;
use std::path::PathBuf;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/markdown-link-display-text/unit-fixture")
        .join(name)
}

fn findings(name: &str) -> Vec<RuleFinding> {
    let root = fixture(name);
    let file = root.join("docs/index.md");
    check_file(&root, &file, &[".md"])
}

#[test]
fn flags_bare_markdown_filename_text_mismatch() {
    let findings = findings("mismatch");

    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].line, 1);
    assert_eq!(findings[0].import.as_deref(), Some("SOURCE-STORIES.md"));
    assert_eq!(
        findings[0].target.as_deref(),
        Some("news-story-clusters.md")
    );
}

#[test]
fn flags_reference_style_markdown_filename_text_mismatch() {
    let findings = findings("reference");

    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].line, 1);
    assert_eq!(findings[0].import.as_deref(), Some("OLD.md"));
    assert_eq!(
        findings[0].target.as_deref(),
        Some("news-story-clusters.md")
    );
}

#[test]
fn flags_collapsed_reference_style_markdown_filename_text_mismatch() {
    let findings = findings("collapsed-reference");

    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].line, 1);
    assert_eq!(findings[0].import.as_deref(), Some("OLD.md"));
    assert_eq!(
        findings[0].target.as_deref(),
        Some("news-story-clusters.md")
    );
}

#[test]
fn flags_shortcut_reference_style_markdown_filename_text_mismatch() {
    let findings = findings("shortcut-reference");

    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].line, 1);
    assert_eq!(findings[0].import.as_deref(), Some("OLD.md"));
    assert_eq!(
        findings[0].target.as_deref(),
        Some("news-story-clusters.md")
    );
}

#[test]
fn keeps_first_duplicate_reference_definition() {
    let findings = findings("duplicate-reference");

    assert!(findings.is_empty(), "{findings:#?}");
}

#[test]
fn handles_reference_style_edge_cases() {
    let links = parser::markdown_links_outside_code(
        "[OLD.md][]\n[OLD2.md][missing]\n[OLD3.md][story]\n\n[]: docs/empty.md\n[story]: <docs/new.md>",
    );

    assert_eq!(links.len(), 1, "{links:#?}");
    assert_eq!(links[0].text, "OLD3.md");
    assert_eq!(links[0].href, "docs/new.md");

    let unclosed_angle =
        parser::markdown_links_outside_code("[OLD.md][story]\n\n[story]: <docs/new.md");
    assert_eq!(unclosed_angle.len(), 1, "{unclosed_angle:#?}");
    assert_eq!(unclosed_angle[0].href, "<docs/new.md");

    assert!(parser::markdown_links_outside_code("[OLD.md][ ]\n\n[OLD.md]: docs/new.md").is_empty());
}

#[test]
fn allows_matching_and_descriptive_links() {
    let findings = findings("allowed");

    assert!(findings.is_empty(), "{findings:#?}");
}

#[test]
fn allows_parenthesized_local_markdown_filenames() {
    let findings = findings("parentheses");

    assert!(findings.is_empty(), "{findings:#?}");
    let (link, _) = parser::parse_inline_link("[x](docs/a\\)b.md)", 0).unwrap();
    assert_eq!(link.href, "docs/a\\)b.md");
    assert!(parser::parse_inline_link("[x](docs/a(b.md)", 0).is_none());
}

#[test]
fn skips_images_non_local_and_directory_hrefs() {
    let findings = findings("skipped");

    assert!(findings.is_empty(), "{findings:#?}");
}

#[test]
fn handles_angle_destinations_and_backticked_text() {
    let findings = findings("angle");

    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].import.as_deref(), Some("OLD-NAME.md"));
    assert_eq!(findings[0].target.as_deref(), Some("new-name.md"));
}

#[test]
fn ignores_links_inside_code() {
    let findings = findings("code");

    assert!(findings.is_empty(), "{findings:#?}");
}

#[test]
fn ignores_links_inside_indented_code() {
    let findings = findings("indented-code");

    assert!(findings.is_empty(), "{findings:#?}");
}

#[test]
fn ignores_links_inside_html_comments() {
    let findings = findings("html-comment");

    assert!(findings.is_empty(), "{findings:#?}");
}

#[test]
fn masks_code_before_html_comments() {
    let findings = findings("html-comment-in-code");

    assert_eq!(findings.len(), 2, "{findings:#?}");
    assert_eq!(findings[0].line, 4);
    assert_eq!(findings[1].line, 6);

    let links = parser::markdown_links_outside_code(
        "``<!-- ` not closed`` [OLD.md](new.md)\n`<!--\n-->` [OLD2.md](new2.md)",
    );
    assert_eq!(links.len(), 2, "{links:#?}");
    assert_eq!(links[0].text, "OLD.md");
    assert_eq!(links[1].text, "OLD2.md");
}

#[test]
fn skips_links_inside_raw_html_blocks() {
    let findings = findings("raw-html");

    assert!(findings.is_empty(), "{findings:#?}");

    let links = parser::markdown_links_outside_code(
        "<PRE class=\"code\">[OLD.md](new.md)</PRE>\n<script\tdefer>[OLD2.md](new2.md)</script>\n<style\n>[OLD3.md](new3.md)</style>",
    );
    assert!(links.is_empty(), "{links:#?}");
}

#[test]
fn allows_percent_encoded_local_basenames() {
    let findings = findings("percent-encoded");

    assert!(findings.is_empty(), "{findings:#?}");
    assert_eq!(href_basename("docs/a%2fb.md").as_deref(), Some("a/b.md"));
    assert_eq!(href_basename("docs/a%2Fb.md").as_deref(), Some("a/b.md"));
    assert_eq!(href_basename("docs/a%zz.md").as_deref(), Some("a%zz.md"));
    assert_eq!(href_basename("docs/a%.md").as_deref(), Some("a%.md"));
}

#[test]
fn preserves_offsets_after_non_ascii_inline_code() {
    let findings = findings("non-ascii-offset");

    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].line, 2);
}

#[test]
fn parses_reference_style_links_outside_code() {
    let links = parser::markdown_links_outside_code(
        "[OLD.md][story]\n\n[story]: docs/news-story-clusters.md",
    );

    assert_eq!(links.len(), 1, "{links:#?}");
    assert_eq!(links[0].text, "OLD.md");
    assert_eq!(links[0].href, "docs/news-story-clusters.md");
}

#[test]
fn parses_nested_and_escaped_brackets_in_inline_links() {
    let links = parser::markdown_links_outside_code(
        "[ADR[1].md](docs/ADR[2].md)\n[ADR\\].md](docs/ADR.md)",
    );

    assert_eq!(links.len(), 2, "{links:#?}");
    assert_eq!(links[0].text, "ADR[1].md");
    assert_eq!(links[0].href, "docs/ADR[2].md");
    assert_eq!(links[1].text, r"ADR\].md");
    assert_eq!(links[1].href, "docs/ADR.md");
}

#[test]
fn handles_unmatched_escaped_and_multi_backtick_inline_code() {
    let findings = findings("inline");

    assert_eq!(findings.len(), 2, "{findings:#?}");
    assert_eq!(findings[0].import.as_deref(), Some("OLD.md"));
    assert_eq!(findings[1].import.as_deref(), Some("OLD2.md"));
}

#[test]
fn skips_escaped_backticks_inside_matched_inline_code() {
    let links = parser::markdown_links_outside_code(
        r#"See `[OLD.md](new.md) \` still code` and [REAL.md](actual.md)"#,
    );

    assert_eq!(links.len(), 1, "{links:#?}");
    assert_eq!(links[0].text, "REAL.md");
}

#[test]
fn long_fences_preserve_offsets_and_ignore_short_inner_fences() {
    let findings = findings("long-fence");

    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].line, 6);
    assert_eq!(findings[0].import.as_deref(), Some("REAL.md"));
}

#[test]
fn tab_indented_fences_do_not_mask_links() {
    let source = "\t```markdown\n[OLD.md](docs/news-story-clusters.md)";
    assert_eq!(parser::strip_fenced_code(source), source);

    let links = parser::markdown_links_outside_code(source);
    assert_eq!(links.len(), 1, "{links:#?}");
    assert_eq!(links[0].text, "OLD.md");

    let findings = findings("tab-indented-fence");

    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].line, 2);
    assert_eq!(findings[0].import.as_deref(), Some("OLD.md"));
}

#[test]
fn reports_real_link_line_after_fenced_content() {
    let findings = findings("after-fence");

    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].line, 5);
    assert_eq!(findings[0].import.as_deref(), Some("REAL.md"));
}

#[test]
fn ignores_links_after_invalid_closing_fence_text() {
    let findings = findings("bad-closing-fence");

    assert!(findings.is_empty(), "{findings:#?}");
    let links = parser::markdown_links_outside_code("    ```\n[REAL.md](actual.md)");
    assert_eq!(links.len(), 1, "{links:#?}");
}

#[test]
fn covers_custom_extensions_non_matching_files_missing_files_and_malformed_links() {
    let root = fixture("custom");
    let mdx = root.join("docs/page.mdx");

    let findings = scan(
        &root,
        &Options {
            extensions: vec![".mdx".to_string()],
        },
        &[mdx.clone(), root.join("docs/missing.mdx")],
    )
    .unwrap();
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].file, "docs/page.mdx");
    assert_eq!(findings[0].import.as_deref(), Some("OLD.mdx"));
    assert_eq!(findings[0].target.as_deref(), Some("new.mdx"));

    assert!(check_file(&root, &mdx, &[".md"]).is_empty());
    assert!(parser::parse_inline_link("[text] no href", 0).is_none());
    assert!(parser::parse_inline_link("[text", 0).is_none());
    assert_eq!(href_destination("<docs/new.md"), "<docs/new.md");
}

#[test]
fn flags_nested_and_escaped_bracket_filename_mismatches() {
    let findings = findings("nested-brackets");

    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].import.as_deref(), Some("ADR[1].md"));
    assert_eq!(findings[0].target.as_deref(), Some("ADR[2].md"));
}
