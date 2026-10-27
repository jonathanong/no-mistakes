use super::*;
use crate::codebase::ts_source::FrozenPathRemapper;
use std::collections::BTreeSet;
use std::path::PathBuf;

// Output cases moved to cli_mrf_snapshot_markdown_child_links. Keep link
// resolution helpers and option defaults below.

fn fixture(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/markdown-child-links/fixture")
            .join(name),
    )
}

#[test]
fn parent_link_resolution_covers_external_absolute_and_dot_paths() {
    let root = fixture("pass");
    let readme = root.join("docs/README.md");
    let guide = root.join("docs/guide.md");
    let known = BTreeSet::from([readme.clone(), guide.clone()]);
    let remapper = FrozenPathRemapper::from_paths([readme.clone(), guide.clone()]);
    let links = super::links::resolve_parent_links(
        &root,
        &readme,
        &[
            String::new(),
            "https://example.com".to_string(),
            "#anchor".to_string(),
            "%2Fguide.md".to_string(),
            "/docs/guide.md".to_string(),
            "./guide.md".to_string(),
            "../docs/guide.md".to_string(),
            "../../outside.md".to_string(),
            "missing.md".to_string(),
            "guide.md".to_string(),
        ],
        &known,
        &remapper,
    );
    assert!(
        links
            .iter()
            .any(|link| link.path == guide && link.whole_file),
        "{links:?}"
    );
}

#[test]
fn normalize_inside_rejects_paths_that_escape_the_root() {
    let root = PathBuf::from("/repo");
    assert_eq!(
        super::links::normalize_inside(&root, &root.join("docs").join("guide.md")),
        Some(root.join("docs").join("guide.md"))
    );
    assert!(super::links::normalize_inside(&root, &root.join("..").join("outside.md")).is_none());
}

#[test]
fn option_defaults_are_empty() {
    assert!(Options::default().groups.is_empty());
    assert!(Group::default().parents.is_empty());
    assert!(!Group::default().count_canonical_html_list_items);
}

#[test]
fn canonical_html_destinations_resolve_like_markdown_links() {
    let root = fixture("canonical-html-pass");
    let readme = root.join("docs/README.md");
    let guide = root.join("docs/guide.md");
    let known = BTreeSet::from([readme.clone(), guide.clone()]);
    let remapper = FrozenPathRemapper::from_paths([readme.clone(), guide.clone()]);
    let dests = super::links::canonical_html_destinations(
        "- <a id=\"guide\"></a>[Guide](guide.md)\n- <a id=\"section\"></a>[Section](guide.md#section)\n",
    );
    let links = super::links::resolve_parent_links(&root, &readme, &dests, &known, &remapper);
    assert!(
        links
            .iter()
            .any(|link| link.path == guide && link.whole_file),
        "{links:?}"
    );
    assert!(
        links
            .iter()
            .any(|link| link.path == guide && !link.whole_file),
        "{links:?}"
    );
}

#[test]
fn canonical_html_destination_parser_skips_urls_and_absolute_paths() {
    assert_eq!(
        super::links::canonical_html_destinations(
            "- <a id=\"guide\"></a>[Guide](guide.md)\n\
             - <a id=\"abs\"></a>[Abs](/docs/guide.md)\n\
             - <a id=\"url\"></a>[Url](https://example.com/guide.md)\n\
             - <a id=\"txt\"></a>[Txt](guide.txt)\n\
             - not a list item [Guide](guide.md)\n"
        ),
        vec!["guide.md".to_string()]
    );
}
