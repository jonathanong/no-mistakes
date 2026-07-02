use super::*;

fn findings(source: &str) -> Vec<RuleFinding> {
    let tmp = tempfile::tempdir().unwrap();
    let file = tmp.path().join("docs/index.md");
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    std::fs::write(&file, source).unwrap();
    check_file(tmp.path(), &file, &[".md"])
}

#[test]
fn flags_bare_markdown_filename_text_mismatch() {
    let findings = findings("[SOURCE-STORIES.md](docs/news-story-clusters.md)\n");

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
    let findings = findings(
        "[target.md](../docs/target.md#section)\n[see the contributing guide](CONTRIBUTING.md)\n[with spaces.md](other.md)\n",
    );

    assert!(findings.is_empty(), "{findings:#?}");
}

#[test]
fn skips_images_non_local_and_directory_hrefs() {
    let findings = findings(
        "![README.md](assets/readme.png)\n[OLD.md](<https://example.com/new.md>)\n[README.md](docs/)\n[README.md](#readme)\n[README.md](mailto:user@example.com)\n",
    );

    assert!(findings.is_empty(), "{findings:#?}");
}

#[test]
fn handles_angle_destinations_and_backticked_text() {
    let findings = findings("[`OLD-NAME.md`](<docs/new-name.md>)\n");

    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].import.as_deref(), Some("OLD-NAME.md"));
    assert_eq!(findings[0].target.as_deref(), Some("new-name.md"));
}

#[test]
fn ignores_links_inside_code() {
    let findings = findings(
        "```markdown\n[OLD-NAME.md](new-name.md)\n```\n~~~\n[OLD2.md](new2.md)\n~~~\nSee `[OLD3.md](new3.md)` for an example.\n",
    );

    assert!(findings.is_empty(), "{findings:#?}");
}

#[test]
fn covers_custom_extensions_non_matching_files_missing_files_and_malformed_links() {
    let tmp = tempfile::tempdir().unwrap();
    let mdx = tmp.path().join("docs/page.mdx");
    std::fs::create_dir_all(mdx.parent().unwrap()).unwrap();
    std::fs::write(
        &mdx,
        "[OLD.md](new.md)\n[unterminated.md\n[reference.md][ref]\n",
    )
    .unwrap();

    let findings = scan(
        tmp.path(),
        &Options {
            extensions: vec![".mdx".to_string()],
        },
        &[mdx.clone(), tmp.path().join("docs/missing.mdx")],
    )
    .unwrap();
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].file, "docs/page.mdx");

    assert!(check_file(tmp.path(), &mdx, &[".md"]).is_empty());
    assert!(parser::parse_inline_link("[text] no href", 0).is_none());
    assert!(parser::parse_inline_link("[text", 0).is_none());
}
