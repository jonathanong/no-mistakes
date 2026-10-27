use super::check_json_snapshot::{assert_saved_fixture, rules_path};
use std::path::PathBuf;

const RULE: &str = "markdown-link-display-text";

fn unit_root(name: &str) -> PathBuf {
    rules_path(RULE, "unit-fixture", name)
}

fn config(name: &str) -> PathBuf {
    rules_path(RULE, "configs", name)
}

#[test]
fn markdown_link_display_text_json_matches_snapshot() {
    let docs = config("docs-index.yml");
    let docs_cases = [
        ("bare_filename_mismatch", "mismatch"),
        ("reference_style", "reference"),
        ("collapsed_reference", "collapsed-reference"),
        ("shortcut_reference", "shortcut-reference"),
        ("duplicate_reference_keeps_first", "duplicate-reference"),
        ("allowed_links", "allowed"),
        ("parenthesized_filename", "parentheses"),
        ("skipped_non_local", "skipped"),
        ("angle_destination", "angle"),
        ("links_inside_code", "code"),
        ("indented_code", "indented-code"),
        ("html_comment", "html-comment"),
        ("html_comment_inside_code", "html-comment-in-code"),
        ("raw_html_pre_script", "raw-html"),
        ("percent_encoded_basename", "percent-encoded"),
        ("non_ascii_offset_line", "non-ascii-offset"),
        ("unmatched_inline_code", "inline"),
        ("long_fence", "long-fence"),
        ("tab_indented_fence", "tab-indented-fence"),
        ("link_after_fence", "after-fence"),
        ("invalid_closing_fence", "bad-closing-fence"),
        ("nested_brackets", "nested-brackets"),
        ("escaped_backtick_in_code", "escaped-backtick-code"),
        ("indented_opening_fence", "indented-opening-fence"),
        ("fence_info_backtick", "fence-info-backtick"),
        ("code_before_html_mask", "code-before-html-mask"),
        ("reference_edge_resolved", "reference-edge-resolved"),
        ("reference_edge_unclosed", "reference-edge-unclosed"),
        ("reference_edge_blank_label", "reference-edge-blank-label"),
        ("escaped_display_match", "escaped-display-match"),
        ("raw_html_tag_blocks", "raw-html-tag-blocks"),
        ("raw_html_multibyte", "raw-html-multibyte"),
        ("unterminated_html_comment", "unterminated-html-comment"),
        ("escaped_closing_backtick", "escaped-closing-backtick"),
        ("escaped_bracket_link", "escaped-bracket-link"),
    ];
    let mut cases: Vec<(&str, PathBuf, PathBuf)> = docs_cases
        .into_iter()
        .map(|(label, name)| (label, unit_root(name), docs.clone()))
        .collect();
    cases.push((
        "custom_mdx_extension",
        unit_root("custom"),
        config("custom-mdx.yml"),
    ));
    cases.push((
        "custom_md_extension_skips_mdx",
        unit_root("custom"),
        config("custom-md.yml"),
    ));
    assert_saved_fixture("markdown_link_display_text", &cases);
}
