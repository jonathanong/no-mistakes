use super::check_json_snapshot::{assert_saved_fixture, rules_path};

const RULE: &str = "no-empty-or-comments-only-files";

fn case(name: &str) -> std::path::PathBuf {
    rules_path(RULE, "fixture", &format!("cases/{name}"))
}

#[test]
fn no_empty_or_comments_only_files_json_matches_snapshot() {
    let ts = rules_path(RULE, "configs", "ts.yml");
    let md = rules_path(RULE, "configs", "md.yml");
    assert_saved_fixture(
        "no_empty_or_comments_only_files",
        &[
            ("pass", rules_path(RULE, "fixture", "pass"), ts.clone()),
            ("fail", rules_path(RULE, "fixture", "fail"), ts.clone()),
            ("sort", rules_path(RULE, "fixture", "sort"), ts.clone()),
            ("empty_file", case("empty-file-detected"), ts.clone()),
            (
                "whitespace_only",
                case("whitespace-only-file-detected"),
                ts.clone(),
            ),
            ("js_line_comment", case("js-line-comment-only"), ts.clone()),
            (
                "js_block_comment",
                case("js-block-comment-only"),
                ts.clone(),
            ),
            (
                "js_real_content",
                case("js-real-content-passes"),
                ts.clone(),
            ),
            (
                "unterminated_block",
                case("unterminated-block-comment-is-comments-only"),
                ts.clone(),
            ),
            ("check_with_files", case("check-with-files-works"), ts),
            (
                "sql_comment",
                case("sql-dash-comment-only"),
                rules_path(RULE, "configs", "sql.yml"),
            ),
            (
                "sql_content",
                case("sql-real-content-passes"),
                rules_path(RULE, "configs", "sql.yml"),
            ),
            ("html_comment", case("html-comment-only"), md.clone()),
            ("html_content", case("html-real-content-passes"), md.clone()),
            ("md_comment", case("md-html-comment-only-flagged"), md),
            (
                "mjs_comment",
                case("mjs-line-comment-only-flagged"),
                rules_path(RULE, "configs", "mjs.yml"),
            ),
            (
                "intentionally_empty",
                case("intentionally-empty-path-is-exempt"),
                rules_path(RULE, "configs", "intentionally-empty.yml"),
            ),
            (
                "default_extensions",
                case("default-extensions-used-when-extensions-empty"),
                rules_path(RULE, "configs", "default-extensions.yml"),
            ),
            (
                "unknown_extension",
                case("unknown-extension-treated-as-raw-content"),
                rules_path(RULE, "configs", "xml.yml"),
            ),
        ],
    );
}
