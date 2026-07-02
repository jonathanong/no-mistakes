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
fn handles_unmatched_escaped_and_multi_backtick_inline_code() {
    let findings = findings("inline");

    assert_eq!(findings.len(), 2, "{findings:#?}");
    assert_eq!(findings[0].import.as_deref(), Some("OLD.md"));
    assert_eq!(findings[1].import.as_deref(), Some("OLD2.md"));
}

#[test]
fn skips_escaped_backticks_inside_matched_inline_code() {
    let links = parser::inline_links_outside_code(
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
    let links = parser::inline_links_outside_code("    ```\n[REAL.md](actual.md)");
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
