use super::{
    discover_files, discover_files_from_visible, discover_source_files,
    discover_source_files_from_visible, discover_visible_paths, format_parse_diagnostic,
    git_visible_files, has_disable_comment, has_disable_file_comment, has_disable_line_comment,
    is_skipped_dir, is_test_file, line_number, normalize_discovery_path, relative_slash_path,
    starts_with_use_client, static_property_key_name, unwrap_ts_wrappers, walk_files,
};
use oxc_allocator::Allocator;
use oxc_ast::ast::{Expression, ObjectPropertyKind, Statement};
use oxc_parser::Parser;
use oxc_span::SourceType;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

mod discovery_preserve;
mod gitignore;
mod source_and_discovery;

#[test]
fn parse_diagnostic_formatter_preserves_diagnostic_and_panic_messages() {
    let path = Path::new("/repo/bad.ts");
    let diagnostics = [oxc_diagnostics::OxcDiagnostic::error("bad syntax")];

    assert_eq!(
        format_parse_diagnostic(path, &diagnostics),
        format!("parsing {}: {:?}", path.display(), diagnostics[0])
    );
    assert_eq!(
        format_parse_diagnostic(path, &[]),
        "parsing /repo/bad.ts: parser panicked without diagnostic details"
    );
}

fn git_init(dir: &Path) {
    let output = Command::new("git")
        .args(["init", "-q", "--initial-branch=main"])
        .current_dir(dir)
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git init failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn git_add_all(dir: &Path) {
    let output = Command::new("git")
        .args(["add", "."])
        .current_dir(dir)
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git add failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn write(dir: &Path, path: &str, content: &str) {
    let full = dir.join(path);
    std::fs::create_dir_all(full.parent().unwrap()).unwrap();
    std::fs::write(full, content).unwrap();
}

fn fixture(path: &str) -> PathBuf {
    let mut parts = path.splitn(3, '/');
    let category = parts.next().unwrap_or(path);
    let sub = parts.next().unwrap_or("");
    let rest = parts.next().unwrap_or("");
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases")
        .join(category)
        .join(sub)
        .join("fixture");
    if !rest.is_empty() {
        p = p.join(rest);
    }
    p
}

// ── has_disable_comment ──────────────────────────────────────────────────

#[test]
fn detected_on_preceding_line() {
    let source = "// no-mistakes-disable-next-line my-rule\nsome code here";
    assert!(has_disable_comment(source, 2, "my-rule"));
}

#[test]
fn detected_on_preceding_hash_comment_line() {
    let source = "# no-mistakes-disable-next-line my-rule\nsome code here";
    assert!(has_disable_comment(source, 2, "my-rule"));
}

#[test]
fn detected_on_preceding_sql_comment_line() {
    let source = "-- no-mistakes-disable-next-line my-rule\nsome code here";
    assert!(has_disable_comment(source, 2, "my-rule"));
}

#[test]
fn not_triggered_on_line_1() {
    let source = "some code here";
    assert!(!has_disable_comment(source, 1, "my-rule"));
}

#[test]
fn wrong_rule_not_matched() {
    let source = "// no-mistakes-disable-next-line other-rule\nsome code here";
    assert!(!has_disable_comment(source, 2, "my-rule"));
}

#[test]
fn matches_with_colon_reason() {
    let source = "// no-mistakes-disable-next-line my-rule: some reason here\nsome code here";
    assert!(has_disable_comment(source, 2, "my-rule"));
}

#[test]
fn matches_with_space_reason() {
    let source = "// no-mistakes-disable-next-line my-rule some reason here\nsome code here";
    assert!(has_disable_comment(source, 2, "my-rule"));
}

#[test]
fn not_matched_when_in_string_literal() {
    let source = "const s = \"no-mistakes-disable-next-line my-rule\"\nsome code here";
    assert!(!has_disable_comment(source, 2, "my-rule"));
}

#[test]
fn not_matched_for_prefix_rule_id() {
    // "my-rule-extra" should NOT match "my-rule" since it's not "my-rule:" or "my-rule "
    // But "my-rule-extra" starts with "my-rule" so we verify the check is correct
    let source = "// no-mistakes-disable-next-line my-rule-extra\nsome code here";
    assert!(!has_disable_comment(source, 2, "my-rule"));
}

#[test]
fn empty_source_returns_false() {
    assert!(!has_disable_comment("", 2, "my-rule"));
}

#[test]
fn disable_next_line_requires_directive_text() {
    let source = "// ordinary comment\nsome code here";
    assert!(!has_disable_comment(source, 2, "my-rule"));
}

// ── has_disable_line_comment ──────────────────────────────────────────────

#[test]
fn disable_line_detected_on_same_line() {
    let source = "some code here // no-mistakes-disable-line my-rule";
    assert!(has_disable_line_comment(source, 1, "my-rule"));
}

#[test]
fn disable_line_detected_on_same_hash_comment_line() {
    let source = "shellcheck issue # no-mistakes-disable-line my-rule";
    assert!(has_disable_line_comment(source, 1, "my-rule"));
}

#[test]
fn disable_line_detected_after_shell_length_expansion() {
    let source = "echo ${#arr[@]} # no-mistakes-disable-line my-rule";
    assert!(has_disable_line_comment(source, 1, "my-rule"));
}

#[test]
fn disable_line_detected_after_shell_operator_hash_comment() {
    let source = "echo ok;# no-mistakes-disable-line my-rule";
    assert!(has_disable_line_comment(source, 1, "my-rule"));
}

#[test]
fn disable_line_detected_on_same_sql_comment_line() {
    let source = "select 1 -- no-mistakes-disable-line my-rule";
    assert!(has_disable_line_comment(source, 1, "my-rule"));
}

#[test]
fn disable_line_detected_on_compact_sql_comment_line() {
    let source = "select 1-- no-mistakes-disable-line my-rule";
    assert!(has_disable_line_comment(source, 1, "my-rule"));
}

#[test]
fn disable_line_detected_after_js_decrement() {
    let source = "i--; // no-mistakes-disable-line my-rule";
    assert!(has_disable_line_comment(source, 1, "my-rule"));
}

#[test]
fn disable_line_detected_after_private_field() {
    let source = "this.#count++; // no-mistakes-disable-line my-rule";
    assert!(has_disable_line_comment(source, 1, "my-rule"));
}

#[test]
fn disable_line_matches_with_reason() {
    let source = "some code here // no-mistakes-disable-line my-rule: intentional";
    assert!(has_disable_line_comment(source, 1, "my-rule"));
}

#[test]
fn disable_line_wrong_rule_not_matched() {
    let source = "some code here // no-mistakes-disable-line other-rule";
    assert!(!has_disable_line_comment(source, 1, "my-rule"));
}

#[test]
fn disable_line_not_matched_for_prefix_rule_id() {
    let source = "some code here // no-mistakes-disable-line my-rule-extra";
    assert!(!has_disable_line_comment(source, 1, "my-rule"));
}

#[test]
fn disable_line_not_matched_when_in_string_literal() {
    let source = "const s = \"// no-mistakes-disable-line my-rule\"";
    assert!(!has_disable_line_comment(source, 1, "my-rule"));
}

#[test]
fn disable_line_handles_escaped_quotes_before_comment() {
    let source = "const s = \"escaped \\\" quote\"; // no-mistakes-disable-line my-rule";
    assert!(has_disable_line_comment(source, 1, "my-rule"));
}

#[test]
fn disable_line_allows_url_before_comment() {
    let source = "const url = \"https://example.com\"; // no-mistakes-disable-line my-rule";
    assert!(has_disable_line_comment(source, 1, "my-rule"));
}

#[test]
fn disable_line_allows_unquoted_url_before_hash_comment() {
    let source = "curl http://example.com # no-mistakes-disable-line my-rule";
    assert!(has_disable_line_comment(source, 1, "my-rule"));
}

#[test]
fn disable_line_allows_unquoted_url_with_double_slash_path() {
    let source = "curl http://example.com/a//b # no-mistakes-disable-line my-rule";
    assert!(has_disable_line_comment(source, 1, "my-rule"));
}

#[test]
fn disable_line_detected_after_colon_context_comment() {
    let source = "case 1: // no-mistakes-disable-line my-rule";
    assert!(has_disable_line_comment(source, 1, "my-rule"));
}

#[test]
fn disable_line_detected_after_regex_literal() {
    let source = r"const re = /a\/\/b/; // no-mistakes-disable-line my-rule";
    assert!(has_disable_line_comment(source, 1, "my-rule"));
}

#[test]
fn disable_line_detected_after_return_regex_literal() {
    let source = r"return /a\/\/b/.test(x); // no-mistakes-disable-line my-rule";
    assert!(has_disable_line_comment(source, 1, "my-rule"));
}

#[test]
fn disable_line_detected_after_arrow_regex_literal() {
    let source = r"const f = () => /a\/\/b/.test(x); // no-mistakes-disable-line my-rule";
    assert!(has_disable_line_comment(source, 1, "my-rule"));
}

#[test]
fn disable_line_detected_after_new_regex_literal() {
    let source = r"const re = new /a\/\/b/; // no-mistakes-disable-line my-rule";
    assert!(has_disable_line_comment(source, 1, "my-rule"));
}

#[test]
fn disable_line_detected_after_regex_character_class() {
    let source = r"const re = /[//]/; // no-mistakes-disable-line my-rule";
    assert!(has_disable_line_comment(source, 1, "my-rule"));
}

#[test]
fn disable_line_ignores_block_comment_text() {
    let source = "const x = 1; /* // no-mistakes-disable-line my-rule */";
    assert!(!has_disable_line_comment(source, 1, "my-rule"));
}

#[test]
fn disable_line_ignores_multiline_block_comment_text() {
    let source = "/* start\ncode // no-mistakes-disable-line my-rule\n*/";
    assert!(!has_disable_line_comment(source, 2, "my-rule"));
}

#[test]
fn disable_line_detected_after_block_comment() {
    let source = "const x = 1; /* block */ // no-mistakes-disable-line my-rule";
    assert!(has_disable_line_comment(source, 1, "my-rule"));
}

#[test]
fn disable_line_detected_after_multiline_block_comment() {
    let source = "/* start\n*/ const x = 1; // no-mistakes-disable-line my-rule";
    assert!(has_disable_line_comment(source, 2, "my-rule"));
}

#[test]
fn disable_line_rejects_zero_line() {
    let source = "some code here // no-mistakes-disable-line my-rule";
    assert!(!has_disable_line_comment(source, 0, "my-rule"));
}

// ── has_disable_file_comment ──────────────────────────────────────────────

#[test]
fn file_disable_detected_in_leading_comment() {
    let source = "// no-mistakes-disable-file my-rule: intentional\nexport const x = 1";
    assert!(has_disable_file_comment(source, "my-rule"));
}

#[test]
fn file_disable_detected_in_leading_hash_comment() {
    let source = "# no-mistakes-disable-file my-rule: intentional\nset -e";
    assert!(has_disable_file_comment(source, "my-rule"));
}

#[test]
fn file_disable_detected_in_leading_sql_comment() {
    let source = "-- no-mistakes-disable-file my-rule: intentional\nselect 1";
    assert!(has_disable_file_comment(source, "my-rule"));
}

#[test]
fn file_disable_skips_leading_blank_lines() {
    let source = "\n\n// no-mistakes-disable-file my-rule\nexport const x = 1";
    assert!(has_disable_file_comment(source, "my-rule"));
}

#[test]
fn file_disable_skips_leading_line_comments() {
    let source =
        "// Copyright 2026\n// eslint-disable no-console\n// no-mistakes-disable-file my-rule\nexport const x = 1";
    assert!(has_disable_file_comment(source, "my-rule"));
}

#[test]
fn file_disable_skips_leading_block_comments() {
    let source =
        "/*\n * Copyright 2026\n */\n// no-mistakes-disable-file my-rule\nexport const x = 1";
    assert!(has_disable_file_comment(source, "my-rule"));
}

#[test]
fn file_disable_skips_long_leading_block_comments() {
    let header = (0..25)
        .map(|i| format!(" * line {i}\n"))
        .collect::<String>();
    let source =
        format!("/*\n{header} */\n// no-mistakes-disable-file my-rule\nexport const x = 1");
    assert!(has_disable_file_comment(&source, "my-rule"));
}

#[test]
fn file_disable_handles_same_line_block_comment_then_directive() {
    let source = "/* Copyright 2026 */ // no-mistakes-disable-file my-rule\nexport const x = 1";
    assert!(has_disable_file_comment(source, "my-rule"));
}

#[test]
fn file_disable_after_block_comment_trailing_code_not_matched() {
    let source = "/* Copyright 2026 */ export const x = 1\n// no-mistakes-disable-file my-rule";
    assert!(!has_disable_file_comment(source, "my-rule"));
}

#[test]
fn file_disable_after_multiline_block_comment_trailing_code_not_matched() {
    let source =
        "/*\n * Copyright 2026\n */ export const x = 1\n// no-mistakes-disable-file my-rule";
    assert!(!has_disable_file_comment(source, "my-rule"));
}

#[test]
fn file_disable_handles_bom() {
    let source = "\u{FEFF}// no-mistakes-disable-file my-rule\nexport const x = 1";
    assert!(has_disable_file_comment(source, "my-rule"));
}

#[test]
fn file_disable_wrong_rule_not_matched() {
    let source = "// no-mistakes-disable-file other-rule\nexport const x = 1";
    assert!(!has_disable_file_comment(source, "my-rule"));
}

#[test]
fn file_disable_matches_with_space_reason() {
    let source = "// no-mistakes-disable-file my-rule because generated\nexport const x = 1";
    assert!(has_disable_file_comment(source, "my-rule"));
}

#[test]
fn file_disable_skips_non_matching_file_directives() {
    let source = "// no-mistakes-disable-file other-rule\n// no-mistakes-disable-file my-rule\nexport const x = 1";
    assert!(has_disable_file_comment(source, "my-rule"));
}

#[test]
fn file_disable_after_code_not_matched() {
    let source = "export const x = 1\n// no-mistakes-disable-file my-rule";
    assert!(!has_disable_file_comment(source, "my-rule"));
}

#[test]
fn file_disable_empty_source_returns_false() {
    assert!(!has_disable_file_comment("", "my-rule"));
}
