use super::check_json_snapshot::{assert_saved_fixture, rules_path};

const RULE: &str = "markdown-child-links";

#[test]
fn markdown_child_links_json_matches_snapshot() {
    let whole_file = rules_path(RULE, "configs", "whole-file.yml");
    let any_link = rules_path(RULE, "configs", "any-link.yml");
    let canonical = rules_path(RULE, "configs", "canonical-html.yml");
    assert_saved_fixture(
        "markdown_child_links",
        &[
            (
                "missing_whole_file",
                rules_path(RULE, "fixture", "fail"),
                whole_file.clone(),
            ),
            (
                "linked_whole_file",
                rules_path(RULE, "fixture", "pass"),
                whole_file.clone(),
            ),
            (
                "fragment_ignored",
                rules_path(RULE, "fixture", "fragment"),
                whole_file,
            ),
            (
                "fragment_counts",
                rules_path(RULE, "fixture", "fragment"),
                any_link.clone(),
            ),
            (
                "missing_markdown_link",
                rules_path(RULE, "fixture", "fail"),
                any_link,
            ),
            (
                "canonical_html_whole_file",
                rules_path(RULE, "fixture", "canonical-html-pass"),
                canonical.clone(),
            ),
            (
                "canonical_html_fragment",
                rules_path(RULE, "fixture", "canonical-html-fragment"),
                canonical,
            ),
        ],
    );
}
