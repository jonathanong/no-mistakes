use super::{
    discover_files, discover_source_files, git_visible_files, has_disable_comment,
    has_disable_file_comment, has_disable_line_comment, is_skipped_dir, is_test_file, line_number,
    normalize_discovery_path, relative_slash_path, starts_with_use_client,
    static_property_key_name, unwrap_ts_wrappers, walk_files,
};
use oxc::allocator::Allocator;
use oxc::ast::ast::{Expression, ObjectPropertyKind, Statement};
use oxc::parser::Parser;
use oxc::span::SourceType;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

mod discovery_preserve;

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

// ── starts_with_use_client ────────────────────────────────────────────────

#[test]
fn detects_single_quote_use_client() {
    assert!(starts_with_use_client(
        "'use client'\nexport function Foo() {}"
    ));
}

#[test]
fn detects_double_quote_use_client() {
    assert!(starts_with_use_client(
        "\"use client\"\nexport function Foo() {}"
    ));
}

#[test]
fn no_match_for_server_component() {
    assert!(!starts_with_use_client("export default function Page() {}"));
}

#[test]
fn only_checks_first_200_bytes() {
    let long_prefix = "a".repeat(210);
    let source = format!("{long_prefix}'use client'");
    assert!(!starts_with_use_client(&source));
}

// ── is_test_file ──────────────────────────────────────────────────────────

#[test]
fn detects_test_suffix_ts() {
    assert!(is_test_file("web/app/foo.test.ts"));
}

#[test]
fn detects_spec_suffix_tsx() {
    assert!(is_test_file("web/app/foo.spec.tsx"));
}

#[test]
fn detects_test_mts() {
    assert!(is_test_file("backend/foo.test.mts"));
}

#[test]
fn detects_tests_directory() {
    assert!(is_test_file("web/app/__tests__/foo.ts"));
}

#[test]
fn non_test_file_not_flagged() {
    assert!(!is_test_file("web/app/page.tsx"));
    assert!(!is_test_file("web/lib/api/server/users.ts"));
}

#[test]
fn source_helpers_cover_paths_lines_wrappers_and_property_names() {
    assert!(is_skipped_dir("node_modules"));
    assert!(!is_skipped_dir("src"));
    assert_eq!(
        relative_slash_path(Path::new("/repo"), Path::new("/repo/src\\file.ts")),
        "src/file.ts"
    );
    assert_eq!(line_number("a\nb\nc", 2), 2);

    let allocator = Allocator::default();
    let parsed = Parser::new(
        &allocator,
        "const x = { plain: (value as string)!, \"quoted\": (<string>value) satisfies string, [dyn]: value };",
        SourceType::ts(),
    )
    .parse();
    let Statement::VariableDeclaration(var_decl) = &parsed.program.body[0] else {
        panic!("expected variable declaration");
    };
    let Expression::ObjectExpression(obj) = var_decl.declarations[0].init.as_ref().expect("init")
    else {
        panic!("expected object");
    };
    let mut names = Vec::new();
    for prop in &obj.properties {
        let ObjectPropertyKind::ObjectProperty(prop) = prop else {
            continue;
        };
        names.push(static_property_key_name(&prop.key));
        let _ = unwrap_ts_wrappers(&prop.value);
    }

    assert_eq!(names, vec![Some("plain"), Some("quoted"), None]);
    assert_eq!(normalize_discovery_path(Path::new("")), Path::new("."));
}

// ── git-aware discovery ─────────────────────────────────────────────────

#[test]
fn git_visible_files_include_tracked_and_untracked_non_ignored_files() {
    let dir = TempDir::new().unwrap();
    git_init(dir.path());
    write(dir.path(), ".gitignore", "dist/\n");
    write(dir.path(), "src/tracked.mts", "");
    write(dir.path(), "dist/ignored.mts", "");
    git_add_all(dir.path());
    write(dir.path(), "src/untracked.mts", "");

    let files = git_visible_files(dir.path()).unwrap();

    assert_eq!(
        files,
        vec![".gitignore", "src/tracked.mts", "src/untracked.mts"]
    );
}

#[test]
fn discover_files_falls_back_outside_git_repositories() {
    let dir = TempDir::new().unwrap();
    write(dir.path(), "src/main.mts", "");

    let files = discover_files(dir.path(), &[]);

    assert_eq!(files, vec![dir.path().join("src/main.mts")]);
}

#[test]
fn fallback_walk_includes_github_workflows() {
    let dir = fixture("ast-snippets/ts-source/hidden-walk");

    let files = walk_files(&dir, &[]);

    assert!(files
        .iter()
        .any(|path| path.ends_with(".github/workflows/ci.yml")));
    assert!(!files
        .iter()
        .any(|path| path.ends_with(".github/workflows/ignored.yml")));
    assert!(files.iter().any(|path| path.ends_with("src/main.mts")));
    assert!(!files.iter().any(|path| path.ends_with(".env")));
    assert!(files
        .iter()
        .any(|path| path.ends_with(".config/secret.mts")));
    assert!(!files
        .iter()
        .any(|path| path.ends_with(".cache/ignored.mts")));

    let files = walk_files(&dir, &[".github".to_string()]);
    assert!(!files
        .iter()
        .any(|path| path.ends_with(".github/workflows/ci.yml")));
}

#[test]
fn fallback_walk_does_not_prune_skipped_named_root() {
    let dir = fixture("ast-snippets/ts-source/dist");

    let files = walk_files(&dir, &[]);

    assert_eq!(files.len(), 1);
    assert!(files
        .iter()
        .any(|path| path.ends_with("dist/root-main.mts")));
}

#[test]
fn discover_source_files_filters_non_ts_js_extensions() {
    let dir = fixture("ast-snippets/ts-source");

    let files = discover_source_files(&dir, &[]);

    assert!(files.iter().any(|path| path.ends_with("jsx-walk-all.tsx")));
    assert!(!files.iter().any(|path| path.ends_with("plain.txt")));
}

#[test]
fn discover_files_normalizes_dot_components() {
    let dir = TempDir::new().unwrap();
    git_init(dir.path());
    write(dir.path(), "src/main.mts", "");
    git_add_all(dir.path());

    let files = discover_files(&dir.path().join("."), &[]);

    assert_eq!(files, vec![dir.path().join("src/main.mts")]);
}

#[test]
fn discover_files_prunes_git_visible_skip_dirs() {
    let dir = TempDir::new().unwrap();
    git_init(dir.path());
    write(dir.path(), "src/main.mts", "");
    write(dir.path(), "node_modules/pkg/index.mts", "");
    write(dir.path(), "dist/bundle.mts", "");
    git_add_all(dir.path());

    let files = discover_files(dir.path(), &[]);

    assert_eq!(files, vec![dir.path().join("src/main.mts")]);
}
