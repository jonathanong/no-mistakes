use std::path::PathBuf;
use std::process::{Command, Output};

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_no-mistakes"))
}

fn fixture(category: &str, scenario: &str) -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/rules")
            .join(category)
            .join(scenario),
    )
}

fn check(root: &PathBuf, yaml: &str) -> Output {
    let config = tempfile::Builder::new().suffix(".yml").tempfile().unwrap();
    std::fs::write(config.path(), yaml).unwrap();
    Command::new(bin())
        .args(["check", "--root"])
        .arg(root)
        .arg("--config")
        .arg(config.path())
        .output()
        .unwrap()
}

fn check_fixture_config(root: &PathBuf, name: &str) -> Output {
    let yaml = std::fs::read_to_string(root.join(name)).unwrap();
    check(root, &yaml)
}

fn stdout(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

fn git(root: &std::path::Path, args: &[&str]) -> bool {
    Command::new("git")
        .args(["-C", root.to_str().unwrap()])
        .args(args)
        .env_remove("GIT_DIR")
        .env_remove("GIT_COMMON_DIR")
        .env_remove("GIT_INDEX_FILE")
        .env_remove("GIT_WORK_TREE")
        .status()
        .unwrap()
        .success()
}

// ── server-route-client-boundary ─────────────────────────────────────────────

#[test]
fn server_route_client_boundary_passes_when_separated() {
    let root = fixture("server-route-client-boundary", "pass");
    let out = check_fixture_config(&root, "check-config.yml");
    assert!(out.status.success(), "exit non-zero: {}", stdout(&out));
}

#[test]
fn server_route_client_boundary_fails_for_client_in_route_folder() {
    let root = fixture("server-route-client-boundary", "fail");
    let out = check_fixture_config(&root, "check-config.yml");
    assert!(!out.status.success(), "expected exit 1");
    assert!(
        stdout(&out).contains("server-route-client-boundary"),
        "{}",
        stdout(&out)
    );
    assert!(
        stdout(&out).contains("backend/api/client.ts"),
        "{}",
        stdout(&out)
    );
}

// ── agents-md-max-size ────────────────────────────────────────────────────────

#[test]
fn agents_md_max_size_passes_under_limits() {
    let root = fixture("agents-md-max-size", "pass");
    let out = check(
        &root,
        "rules:\n  - rule: agents-md-max-size\n    scope: repository\n    options:\n      maxLines: 5\n      maxChars: 1000\n",
    );
    assert!(out.status.success(), "exit non-zero: {}", stdout(&out));
    assert!(
        stdout(&out).is_empty() || !stdout(&out).contains("lines"),
        "{}",
        stdout(&out)
    );
}

#[test]
fn agents_md_max_size_fails_over_line_limit() {
    let root = fixture("agents-md-max-size", "fail");
    let out = check(
        &root,
        "rules:\n  - rule: agents-md-max-size\n    scope: repository\n    options:\n      maxLines: 2\n",
    );
    assert!(!out.status.success(), "expected exit 1 for over-limit file");
    assert!(stdout(&out).contains("3 lines"), "{}", stdout(&out));
    assert!(stdout(&out).contains("CLAUDE.md"), "{}", stdout(&out));
}

#[test]
fn agents_md_max_size_json_output_includes_rule_id() {
    let root = fixture("agents-md-max-size", "fail");
    // Re-run with --format json
    let config = tempfile::Builder::new().suffix(".yml").tempfile().unwrap();
    std::fs::write(
        config.path(),
        "rules:\n  - rule: agents-md-max-size\n    scope: repository\n    options:\n      maxLines: 2\n",
    )
    .unwrap();
    let out_json = Command::new(bin())
        .args(["check", "--root"])
        .arg(&root)
        .arg("--config")
        .arg(config.path())
        .args(["--format", "json"])
        .output()
        .unwrap();
    let body = stdout(&out_json);
    assert!(
        body.contains("agents-md-max-size"),
        "rule id missing: {body}"
    );
    assert!(!out_json.status.success());
}

#[test]
fn agents_md_max_size_disabled_skips_check() {
    let root = fixture("agents-md-max-size", "fail");
    let out = check(
        &root,
        "rules:\n  - rule: agents-md-max-size\n    enabled: false\n    scope: repository\n    options:\n      maxLines: 2\n",
    );
    assert!(
        out.status.success(),
        "disabled rule should not fail: {}",
        stdout(&out)
    );
}

// ── rust-max-lines-per-file ───────────────────────────────────────────────────

#[test]
fn rust_max_lines_per_file_passes_under_limit() {
    let root = fixture("rust-max-lines-per-file", "pass");
    let out = check(
        &root,
        "rules:\n  - rule: rust-max-lines-per-file\n    scope: repository\n    options:\n      srcMax: 20\n",
    );
    assert!(out.status.success(), "exit non-zero: {}", stdout(&out));
}

#[test]
fn rust_max_lines_per_file_fails_over_limit() {
    let root = fixture("rust-max-lines-per-file", "fail");
    let out = check(
        &root,
        "rules:\n  - rule: rust-max-lines-per-file\n    scope: repository\n    options:\n      srcMax: 3\n",
    );
    assert!(!out.status.success(), "expected exit 1");
    assert!(stdout(&out).contains("code lines"), "{}", stdout(&out));
    assert!(stdout(&out).contains("big.rs"), "{}", stdout(&out));
}

#[test]
fn rust_max_lines_per_file_disabled_skips() {
    let root = fixture("rust-max-lines-per-file", "fail");
    let out = check(
        &root,
        "rules:\n  - rule: rust-max-lines-per-file\n    enabled: false\n    scope: repository\n    options:\n      srcMax: 3\n",
    );
    assert!(
        out.status.success(),
        "disabled rule must not fail: {}",
        stdout(&out)
    );
}

#[test]
fn rust_max_lines_per_file_json_has_rule_id() {
    let root = fixture("rust-max-lines-per-file", "fail");
    let config = tempfile::Builder::new().suffix(".yml").tempfile().unwrap();
    std::fs::write(
        config.path(),
        "rules:\n  - rule: rust-max-lines-per-file\n    scope: repository\n    options:\n      srcMax: 3\n",
    )
    .unwrap();
    let out = Command::new(bin())
        .args(["check", "--root"])
        .arg(&root)
        .arg("--config")
        .arg(config.path())
        .args(["--format", "json"])
        .output()
        .unwrap();
    assert!(
        stdout(&out).contains("rust-max-lines-per-file"),
        "{}",
        stdout(&out)
    );
}

// ── rust-no-inline-tests ──────────────────────────────────────────────────────

#[test]
fn rust_no_inline_tests_passes_out_of_line() {
    let root = fixture("rust-no-inline-tests", "pass");
    let out = check(
        &root,
        "rules:\n  - rule: rust-no-inline-tests\n    scope: repository\n",
    );
    assert!(out.status.success(), "exit non-zero: {}", stdout(&out));
}

#[test]
fn rust_no_inline_tests_fails_inline_block() {
    let root = fixture("rust-no-inline-tests", "fail");
    let out = check(
        &root,
        "rules:\n  - rule: rust-no-inline-tests\n    scope: repository\n",
    );
    assert!(!out.status.success(), "expected exit 1");
    assert!(stdout(&out).contains("inline"), "{}", stdout(&out));
    assert!(stdout(&out).contains("lib.rs"), "{}", stdout(&out));
}

#[test]
fn rust_no_inline_tests_fails_cfg_test_helper_item() {
    let root = fixture("rust-no-inline-tests", "fail-helper");
    let out = check(
        &root,
        "rules:\n  - rule: rust-no-inline-tests\n    scope: repository\n",
    );
    assert!(!out.status.success(), "expected exit 1");
    assert!(
        stdout(&out).contains("inline #[cfg(test)] item"),
        "{}",
        stdout(&out)
    );
    assert!(stdout(&out).contains("lib.rs"), "{}", stdout(&out));
}

#[test]
fn rust_no_inline_tests_disabled_skips() {
    let root = fixture("rust-no-inline-tests", "fail");
    let out = check(
        &root,
        "rules:\n  - rule: rust-no-inline-tests\n    enabled: false\n    scope: repository\n",
    );
    assert!(
        out.status.success(),
        "disabled rule must not fail: {}",
        stdout(&out)
    );
}

#[test]
fn rust_no_inline_tests_json_has_rule_id() {
    let root = fixture("rust-no-inline-tests", "fail");
    let config = tempfile::Builder::new().suffix(".yml").tempfile().unwrap();
    std::fs::write(
        config.path(),
        "rules:\n  - rule: rust-no-inline-tests\n    scope: repository\n",
    )
    .unwrap();
    let out = Command::new(bin())
        .args(["check", "--root"])
        .arg(&root)
        .arg("--config")
        .arg(config.path())
        .args(["--format", "json"])
        .output()
        .unwrap();
    assert!(
        stdout(&out).contains("rust-no-inline-tests"),
        "{}",
        stdout(&out)
    );
    assert!(!out.status.success());
}

// ── rust-no-inline-allows ─────────────────────────────────────────────────────

#[test]
fn rust_no_inline_allows_passes_without_allow_attributes() {
    let root = fixture("rust-no-inline-allows", "pass");
    let out = check(
        &root,
        "rules:\n  - rule: rust-no-inline-allows\n    scope: repository\n",
    );
    assert!(out.status.success(), "exit non-zero: {}", stdout(&out));
}

#[test]
fn rust_no_inline_allows_fails_inline_allow() {
    let root = fixture("rust-no-inline-allows", "fail");
    let out = check(
        &root,
        "rules:\n  - rule: rust-no-inline-allows\n    scope: repository\n",
    );
    assert!(!out.status.success(), "expected exit 1");
    assert!(
        stdout(&out).contains("allow(dead_code)"),
        "{}",
        stdout(&out)
    );
    assert!(stdout(&out).contains("lib.rs"), "{}", stdout(&out));
}

#[test]
fn rust_no_inline_allows_filesystem_runner_discovers_files() {
    let root = fixture("rust-no-inline-allows", "fail");
    let config = tempfile::Builder::new().suffix(".yml").tempfile().unwrap();
    std::fs::write(
        config.path(),
        "rules:\n  - rule: rust-no-inline-allows\n    scope: repository\n",
    )
    .unwrap();

    let findings =
        no_mistakes::codebase::rules::run_filesystem_rules(&root, Some(config.path())).unwrap();

    assert!(findings
        .iter()
        .any(|finding| finding.rule == no_mistakes::codebase::rules::RUST_NO_INLINE_ALLOWS));
}

#[test]
fn rust_no_inline_allows_filesystem_runner_accepts_absolute_roots() {
    let fixture_root = fixture("rust-no-inline-allows", "absolute-roots");
    let src = fixture_root.join("src");
    let config = tempfile::Builder::new().suffix(".yml").tempfile().unwrap();
    let template = std::fs::read_to_string(fixture_root.join(".no-mistakes.yml.in")).unwrap();
    std::fs::write(
        config.path(),
        template.replace("__ROOT__", &src.to_string_lossy()),
    )
    .unwrap();

    let findings =
        no_mistakes::codebase::rules::run_filesystem_rules(&fixture_root, Some(config.path()))
            .unwrap();

    assert_eq!(findings.len(), 2);
    assert_eq!(findings[0].file, "src/a.rs");
    assert_eq!(findings[1].file, "src/b.rs");
}

// ── gitignored files are skipped ─────────────────────────────────────────────

#[test]
fn agents_md_max_size_skips_gitignored_files() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();

    // Over-limit file in a gitignored directory
    std::fs::create_dir_all(root.join("ignored")).unwrap();
    let big: String = "line\n".repeat(300);
    std::fs::write(root.join("ignored/AGENTS.md"), &big).unwrap();

    // Passing tracked file
    std::fs::write(root.join("CLAUDE.md"), "# ok\n").unwrap();

    // .gitignore excludes the directory
    std::fs::write(root.join(".gitignore"), "ignored/\n").unwrap();

    // Initialise a git repo and commit so git ls-files is the source of truth
    assert!(git(root, &["init", "-q"]));
    assert!(git(root, &["add", "."]));
    assert!(git(
        root,
        &[
            "-c",
            "user.email=t@t",
            "-c",
            "user.name=t",
            "commit",
            "-qm",
            "init"
        ]
    ));

    let config = tempfile::Builder::new().suffix(".yml").tempfile().unwrap();
    std::fs::write(
        config.path(),
        "rules:\n  - rule: agents-md-max-size\n    scope: repository\n    options:\n      maxLines: 5\n",
    )
    .unwrap();
    let out = Command::new(bin())
        .args(["check", "--root"])
        .arg(root)
        .arg("--config")
        .arg(config.path())
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "gitignored files must not be flagged: {}",
        stdout(&out)
    );
}

#[test]
fn rust_no_inline_tests_skips_gitignored_files() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();

    std::fs::create_dir_all(root.join("generated")).unwrap();
    std::fs::write(
        root.join("generated/lib.rs"),
        "#[cfg(test)]\nmod tests {\n}\n",
    )
    .unwrap();
    std::fs::write(root.join("clean.rs"), "pub fn ok() {}\n").unwrap();
    std::fs::write(root.join(".gitignore"), "generated/\n").unwrap();

    assert!(git(root, &["init", "-q"]));
    assert!(git(root, &["add", "."]));
    assert!(git(
        root,
        &[
            "-c",
            "user.email=t@t",
            "-c",
            "user.name=t",
            "commit",
            "-qm",
            "init"
        ]
    ));

    let config = tempfile::Builder::new().suffix(".yml").tempfile().unwrap();
    std::fs::write(
        config.path(),
        "rules:\n  - rule: rust-no-inline-tests\n    scope: repository\n",
    )
    .unwrap();
    let out = Command::new(bin())
        .args(["check", "--root"])
        .arg(root)
        .arg("--config")
        .arg(config.path())
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "gitignored files must not be flagged: {}",
        stdout(&out)
    );
}
