use super::link_target::{finding_for_link, href_basename, href_destination};
use super::*;
use std::path::PathBuf;

// Output cases moved to cli_mrf_snapshot_markdown_link_display_text. Keep
// parser details check JSON cannot show and unreadable-file errors below.

fn scan(
    root: &std::path::Path,
    opts: &Options,
    files: &[PathBuf],
) -> anyhow::Result<Vec<RuleFinding>> {
    let sources = crate::codebase::rules::source_store_for_files(files);
    let mut plan = super::super::markdown_facts::MarkdownFactPlan::default();
    plan.request_display_links(files.iter().cloned());
    let facts = super::super::markdown_facts::MarkdownFactMap::prepare(&plan, &sources);
    scan_with_facts(root, opts, files, &facts)
}

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/markdown-link-display-text/unit-fixture")
        .join(name)
}

#[test]
fn crlf_raw_html_tag_still_hides_the_following_link() {
    // A Windows checkout can rewrite the saved fixture's LF as CRLF. The tag
    // still opens the raw block, matching the LF snapshot.
    let style = "<style\r\n>[OLD3.md](new3.md)</style>\r\n";
    let script = "<script\r\n[OLD2.md](docs/news-story-clusters.md)\r\n</script>\r\n";
    assert!(parser::markdown_links_outside_code(style).is_empty());
    assert!(parser::markdown_links_outside_code(script).is_empty());
}

#[test]
fn reference_style_edge_hrefs_keep_angle_and_unclosed_destinations() {
    let links = parser::markdown_links_outside_code(
        "[OLD.md][]\n[OLD2.md][missing]\n[OLD3.md][story]\n\n[]: docs/empty.md\n[story]: <docs/new.md>",
    );
    assert_eq!(links[0].href, "docs/new.md");

    let unclosed_angle =
        parser::markdown_links_outside_code("[OLD.md][story]\n\n[story]: <docs/new.md");
    assert_eq!(unclosed_angle[0].href, "<docs/new.md");
}

#[test]
fn escaped_filename_text_matches_after_unescape() {
    assert!(finding_for_link(
        "docs/index.md",
        "[ADR\\.md](docs/ADR.md)",
        parser::parse_inline_link("[ADR\\.md](docs/ADR.md)", 0)
            .unwrap()
            .0,
        &[".md"],
    )
    .is_none());
    assert!(finding_for_link(
        "docs/index.md",
        "[ADR\\[1\\].md](docs/ADR[1].md)",
        parser::parse_inline_link("[ADR\\[1\\].md](docs/ADR[1].md)", 0)
            .unwrap()
            .0,
        &[".md"],
    )
    .is_none());
    assert!(finding_for_link(
        "docs/index.md",
        "[ADR\\`.md](docs/ADR.md)",
        parser::parse_inline_link("[ADR\\`.md](docs/ADR.md)", 0)
            .unwrap()
            .0,
        &[".md"],
    )
    .is_none());
}

#[test]
fn allows_parenthesized_local_markdown_filenames() {
    let (link, _) = parser::parse_inline_link("[x](docs/a\\)b.md)", 0).unwrap();
    assert_eq!(link.href, "docs/a\\)b.md");
    assert_eq!(href_basename("docs/a\\)b.md").as_deref(), Some("a)b.md"));
    assert_eq!(href_basename("docs/a\\x.md").as_deref(), Some("a\\x.md"));
    assert_eq!(href_basename("docs/a\\").as_deref(), Some("a\\"));
    let (angle_link, _) = parser::parse_inline_link("[a)b.md](<docs/a)b.md>)", 0).unwrap();
    assert_eq!(angle_link.href, "<docs/a)b.md>");
    let (titled_angle_link, _) =
        parser::parse_inline_link("[OLD.md](<docs/new.md> \"details\")", 0).unwrap();
    assert_eq!(titled_angle_link.href, "<docs/new.md> \"details\"");
    let (escaped_title_link, _) =
        parser::parse_inline_link("[OLD.md](<docs/new.md> \"detail\\)s\")", 0).unwrap();
    assert_eq!(escaped_title_link.href, "<docs/new.md> \"detail\\)s\"");
    let (paren_title_link, _) =
        parser::parse_inline_link("[OLD.md](docs/new.md \"see (details\")", 0).unwrap();
    assert_eq!(paren_title_link.href, "docs/new.md \"see (details\"");
    let (escaped_paren_title_link, _) =
        parser::parse_inline_link("[OLD.md](docs/new.md \"see \\(details\")", 0).unwrap();
    assert_eq!(
        escaped_paren_title_link.href,
        "docs/new.md \"see \\(details\""
    );
    assert!(parser::parse_inline_link("[x](docs/a(b.md)", 0).is_none());
    assert!(parser::parse_inline_link("[x](<docs/a.md> \"title\"", 0).is_none());
    assert!(parser::parse_inline_link("[x](docs/a.md \"title", 0).is_none());
}

#[test]
fn preserves_multibyte_raw_html_link_offset() {
    let source = "<pre>日本語</pre>\n[OLD.md](new.md)";
    let links = parser::markdown_links_outside_code(source);
    assert_eq!(links[0].offset, source.find("[OLD.md]").unwrap());
}

#[test]
fn allows_percent_encoded_local_basenames() {
    assert_eq!(href_basename("docs/a%2fb.md").as_deref(), Some("a/b.md"));
    assert_eq!(href_basename("docs/a%2Fb.md").as_deref(), Some("a/b.md"));
    assert_eq!(href_basename("docs/a%zz.md").as_deref(), Some("a%zz.md"));
    assert_eq!(href_basename("docs/a%.md").as_deref(), Some("a%.md"));
}

#[test]
fn preserves_offsets_after_non_ascii_inline_code() {
    let source = "```\né\n```\n[OLD.md](docs/new.md)";
    let links = parser::markdown_links_outside_code(source);
    assert_eq!(links[0].offset, source.find("[OLD.md]").unwrap());
}

#[test]
fn parses_reference_style_links_outside_code() {
    let links = parser::markdown_links_outside_code(
        "[OLD.md][story]\n\n[story]: docs/news-story-clusters.md",
    );
    assert_eq!(links[0].href, "docs/news-story-clusters.md");
}

#[test]
fn parses_nested_and_escaped_brackets_in_inline_links() {
    let links = parser::markdown_links_outside_code(
        "[ADR[1].md](docs/ADR[2].md)\n[ADR\\].md](docs/ADR.md)",
    );

    assert_eq!(links.len(), 2, "{links:#?}");
    assert_eq!(links[0].href, "docs/ADR[2].md");
    assert_eq!(links[1].text, r"ADR\].md");
    assert_eq!(links[1].href, "docs/ADR.md");
}

#[test]
fn tab_indented_fences_do_not_change_the_source() {
    let source = "\t```markdown\n[OLD.md](docs/news-story-clusters.md)";
    assert_eq!(parser::strip_fenced_code(source), source);
}

#[test]
fn malformed_links_do_not_parse() {
    assert!(parser::parse_inline_link("[text] no href", 0).is_none());
    assert!(parser::parse_inline_link("[text", 0).is_none());
    assert_eq!(href_destination("<docs/new.md"), "<docs/new.md");
}

#[test]
fn reports_missing_configured_extension_sources() {
    let root = fixture("custom");
    let missing = root.join("docs/missing.mdx");

    let error = scan(
        &root,
        &Options {
            extensions: vec![".mdx".to_string()],
        },
        &[missing],
    )
    .unwrap_err();

    assert!(error.to_string().contains(RULE_ID));
    assert!(error.to_string().contains("could not read Markdown file"));
}
