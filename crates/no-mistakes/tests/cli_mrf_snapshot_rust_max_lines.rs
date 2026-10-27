use super::check_json_snapshot::{assert_saved_fixture, rules_path};
use std::path::PathBuf;

const RULE: &str = "rust-max-lines-per-file";

#[test]
fn rust_max_lines_per_file_json_matches_snapshot() {
    // srcMax is the code-line count minus one, so the message contains that
    // count. Count 0 uses srcMax 0 and stays clean. physical_at_code_limit is
    // clean at 3 code lines; physical_over_code_limit fails at srcMax 2.
    let mut cases = [
        ("within_src_limit", "within-src-limit", "within-src.yml"),
        ("over_src_limit", "over-src-limit", "over-src.yml"),
        ("sort_by_file", "sort-by-file", "sort-by-file.yml"),
        ("test_file_limit", "test-limit", "test-limit.yml"),
        ("tests_directory", "tests-dir", "tests-dir.yml"),
        ("excludes", "respects-excludes", "excludes.yml"),
        ("relative_roots", "roots", "roots-sub.yml"),
        ("disable_file", "disable-file", "disable-file.yml"),
        ("physical_over_code_limit", "unit", "physical-max-2.yml"),
        ("physical_at_code_limit", "unit", "physical-max-3.yml"),
        ("line_count_empty", "line-counts", "line-count-empty.yml"),
        (
            "line_count_blank_and_comment",
            "line-counts",
            "line-count-blank-and-comment.yml",
        ),
        ("line_count_simple", "line-counts", "line-count-simple.yml"),
        (
            "line_count_block_comment",
            "line-counts",
            "line-count-block-comment.yml",
        ),
        (
            "line_count_nested_block",
            "line-counts",
            "line-count-nested-block.yml",
        ),
        (
            "line_count_inline_block",
            "line-counts",
            "line-count-inline-block.yml",
        ),
        (
            "line_count_doc_comment",
            "line-counts",
            "line-count-doc-comment.yml",
        ),
        (
            "line_count_string_block_marker",
            "line-counts",
            "line-count-string-block.yml",
        ),
        (
            "line_count_string_line_marker",
            "line-counts",
            "line-count-string-line.yml",
        ),
        (
            "line_count_escaped_quote",
            "line-counts",
            "line-count-escaped-quote.yml",
        ),
        ("line_count_raw_string", "unit", "line-count-raw-string.yml"),
        (
            "line_count_multiline_raw_string",
            "unit",
            "line-count-multiline-raw.yml",
        ),
        (
            "line_count_byte_raw_string",
            "unit",
            "line-count-byte-raw.yml",
        ),
        (
            "line_count_char_slash",
            "line-counts",
            "line-count-char-slash.yml",
        ),
        (
            "line_count_escaped_char",
            "line-counts",
            "line-count-escaped-char.yml",
        ),
    ]
    .map(|(label, fixture, config)| {
        (
            label,
            rules_path(RULE, "fixture", fixture),
            rules_path(RULE, "configs", config),
        )
    })
    .to_vec();
    // No repository scope: each application must keep its own project root.
    cases.push((
        "per_project",
        no_mistakes::codebase::ts_resolver::normalize_path(
            &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../test-cases/rules/multiple-rule-applications/fixture"),
        ),
        rules_path(RULE, "configs", "per-project.yml"),
    ));
    assert_saved_fixture("rust_max_lines_per_file", &cases);
}
