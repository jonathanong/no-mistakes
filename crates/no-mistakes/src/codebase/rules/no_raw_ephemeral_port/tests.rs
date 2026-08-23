use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::{Path, PathBuf};

fn fixture(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/no-raw-ephemeral-port/fixture")
            .join(name),
    )
}

fn config(yaml: &str) -> NoMistakesConfig {
    NoMistakesConfig {
        rules: vec![RuleDef {
            rule: RULE_ID.to_string(),
            scope: Some(RuleScope::Repository),
            options: serde_yaml::from_str(yaml).unwrap(),
            ..Default::default()
        }],
        ..Default::default()
    }
}

fn run(root: &Path, yaml: &str) -> Vec<RuleFinding> {
    let files = crate::codebase::ts_source::discover_files(root, &[]);
    check_with_files(root, &config(yaml), &files).unwrap()
}

fn run_deferred(root: &Path, yaml: &str, defer_suppression: bool) -> Vec<RuleFinding> {
    let files = crate::codebase::ts_source::discover_files(root, &[]);
    let sources = super::super::source_store_for_files(&files);
    check_with_files_sources_and_deferred_suppression(
        root,
        &config(yaml),
        &files,
        &sources,
        defer_suppression,
    )
    .unwrap()
}

fn bind_lines(source: &str) -> Vec<usize> {
    python::scan_lines(source, &regex::Regex::new(python::BIND_PATTERN).unwrap())
}

#[test]
fn python_tuple_bind_zero_is_a_finding() {
    let findings = run(&fixture("fail-python"), "{}");
    assert!(
        findings
            .iter()
            .any(|finding| finding.file == "server.py" && finding.line == 3),
        "{findings:?}"
    );
    assert!(
        findings
            .iter()
            .any(|finding| finding.file == "server.py" && finding.line == 4),
        "{findings:?}"
    );
    assert!(
        findings
            .iter()
            .all(|finding| finding.message == DEFAULT_MESSAGE),
        "{findings:?}"
    );
}

#[test]
fn shell_and_yaml_tuple_binds_are_findings() {
    let findings = run(&fixture("fail-python"), "{}");
    assert!(
        findings.iter().any(|finding| finding.file == "script.sh"),
        "{findings:?}"
    );
    assert!(
        findings.iter().any(|finding| finding.file == "compose.yml"),
        "{findings:?}"
    );
}

#[test]
fn listen_zero_forms_are_findings() {
    let findings = run(&fixture("fail-listen"), "{}");
    let files: Vec<_> = findings
        .iter()
        .map(|finding| finding.file.as_str())
        .collect();
    assert!(files.contains(&"server.ts"), "{findings:?}");
    assert!(files.contains(&"options.ts"), "{findings:?}");
    assert!(files.contains(&"trailing.ts"), "{findings:?}");
    assert!(files.contains(&"app.jsx"), "{findings:?}");
}

#[test]
fn non_zero_binds_and_listens_pass() {
    assert!(run(&fixture("pass"), "{}").is_empty());
}

#[test]
fn allowlisted_binder_is_skipped() {
    let findings = run(&fixture("allow"), "allow: [\"binder.ts\"]\n");
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn custom_message_is_appended() {
    let findings = run(
        &fixture("fail-listen"),
        "message: use a configured allocator\n",
    );
    assert!(
        findings
            .iter()
            .all(|finding| finding.message.ends_with("use a configured allocator")),
        "{findings:?}"
    );
    assert!(
        findings
            .iter()
            .all(|finding| finding.message.starts_with(DEFAULT_MESSAGE)),
        "{findings:?}"
    );
}

#[test]
fn custom_include_skips_other_extensions() {
    let findings = run(&fixture("fail-listen"), "include: [\"**/*.py\"]\n");
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn include_without_matches_is_silent() {
    assert!(run(&fixture("fail-python"), "include: [\"nope.py\"]\n").is_empty());
}

#[test]
fn disable_file_comment_skips_the_source() {
    assert!(run(&fixture("disable"), "{}").is_empty());
}

#[test]
fn unreadable_file_is_skipped() {
    let root = fixture("pass");
    let path = root.join("missing.py");
    let sources = super::super::source_store_for_files(&[]);
    let opts = compile_options(Options::default()).unwrap();
    assert!(scan::check_file(&root, &path, &opts, &sources, false).is_empty());
}

#[test]
fn bind_regex_accepts_whitespace_and_single_quotes() {
    assert_eq!(bind_lines("sock.bind(('127.0.0.1', 0))\n"), vec![1]);
    assert_eq!(bind_lines(". bind ( ( \"x\" , 0 ) )\n"), vec![1]);
    assert!(bind_lines("sock.bind((\"127.0.0.1\", 8080))\n").is_empty());
    assert!(bind_lines("sock.bind(0)\n").is_empty());
}

#[test]
fn bind_regex_accepts_python_integer_zero_literals() {
    for source in [
        "sock.bind((host, 0))\n",
        "sock.bind((host, 00))\n",
        "sock.bind((host, 0_0))\n",
        "sock.bind((host, 0x0))\n",
        "sock.bind((host, 0X0))\n",
        "sock.bind((host, 0o0))\n",
        "sock.bind((host, 0O0))\n",
        "sock.bind((host, 0b0))\n",
        "sock.bind((host, 0B0))\n",
        "sock.bind((\"::1\", 0x0, 0, 0))\n",
    ] {
        assert_eq!(bind_lines(source), vec![1], "{source}");
    }
    assert!(bind_lines("sock.bind((host, 0x1))\n").is_empty());
    assert!(bind_lines("sock.bind((host, 0o7))\n").is_empty());
    assert!(bind_lines("sock.bind((host, 0b1))\n").is_empty());
    assert!(bind_lines("sock.bind((host, 10))\n").is_empty());
    assert!(bind_lines("sock.bind((host, 0.0))\n").is_empty());
}

#[test]
fn bind_regex_accepts_non_literal_hosts() {
    assert_eq!(bind_lines("sock.bind((host, 0))\n"), vec![1]);
    assert_eq!(bind_lines("sock.bind((get_host(), 0))\n"), vec![1]);
    assert!(bind_lines("sock.bind((host, port))\n").is_empty());
}

#[test]
fn bind_regex_accepts_ipv6_four_tuples() {
    assert_eq!(bind_lines("sock.bind((\"::1\", 0, 0, 0))\n"), vec![1]);
    assert_eq!(bind_lines("sock.bind(('::1', 0, 0, 0))\n"), vec![1]);
    assert_eq!(bind_lines(".bind( ( '::1' , 0 , 0 , 0 ) )\n"), vec![1]);
    assert!(bind_lines("sock.bind((\"::1\", 8080, 0, 0))\n").is_empty());
}

#[test]
fn bind_regex_skips_comment_lines_but_keeps_command_strings() {
    assert!(bind_lines("# sock.bind((\"127.0.0.1\", 0))\n").is_empty());
    assert!(bind_lines("  // sock.bind((\"127.0.0.1\", 0))\n").is_empty());
    assert_eq!(
        bind_lines("command: python -c 's.bind((\"127.0.0.1\", 0))'\n"),
        vec![1]
    );
    assert_eq!(
        bind_lines("python -c 's.bind((\"0.0.0.0\", 0))'\n"),
        vec![1]
    );
}

#[test]
fn listen_numeric_zero_and_object_port_are_flagged() {
    assert_eq!(
        ast::scan_lines(Path::new("server.ts"), "server.listen(0);\n"),
        vec![1]
    );
    assert_eq!(
        ast::scan_lines(Path::new("server.ts"), "server.listen(0, '127.0.0.1');\n"),
        vec![1]
    );
    assert_eq!(
        ast::scan_lines(Path::new("server.ts"), "server.listen({ port: 0 });\n"),
        vec![1]
    );
    assert_eq!(
        ast::scan_lines(
            Path::new("server.ts"),
            "server.listen({ host: '127.0.0.1', port: 0 }, () => {});\n"
        ),
        vec![1]
    );
    assert_eq!(
        ast::scan_lines(Path::new("server.ts"), "server.listen({ 'port': 0 });\n"),
        vec![1]
    );
    assert_eq!(
        ast::scan_lines(Path::new("server.ts"), r#"server["listen"](0);"#),
        vec![1]
    );
    assert_eq!(
        ast::scan_lines(Path::new("server.ts"), "server.listen(+0);\n"),
        vec![1]
    );
    assert_eq!(
        ast::scan_lines(Path::new("server.ts"), "server.listen(-0);\n"),
        vec![1]
    );
    assert_eq!(
        ast::scan_lines(Path::new("server.ts"), "server.listen({ port: -0 });\n"),
        vec![1]
    );
    assert_eq!(
        ast::scan_lines(Path::new("server.ts"), "server.listen({ port: +0 });\n"),
        vec![1]
    );
}

#[test]
fn listen_non_literal_ports_are_ignored() {
    assert!(ast::scan_lines(Path::new("server.ts"), "server.listen(8080);\n").is_empty());
    assert!(ast::scan_lines(Path::new("server.ts"), "server.listen({ port: 8080 });\n").is_empty());
    assert!(ast::scan_lines(Path::new("server.ts"), "server.listen(port);\n").is_empty());
    assert!(ast::scan_lines(Path::new("server.ts"), "server.listen({ port });\n").is_empty());
    assert!(ast::scan_lines(Path::new("server.ts"), "server.listen({ port: '0' });\n").is_empty());
    assert!(ast::scan_lines(Path::new("server.ts"), "listen(0);\n").is_empty());
    assert!(ast::scan_lines(Path::new("server.ts"), "server.listen({ ...opts });\n").is_empty());
    assert!(ast::scan_lines(Path::new("server.py"), "server.listen(0);\n").is_empty());
}

#[test]
fn jsx_and_cjs_listen_zero_are_flagged() {
    assert_eq!(
        ast::scan_lines(
            Path::new("app.tsx"),
            "export const A = () => { s.listen(0); return <div/> }\n"
        ),
        vec![1]
    );
    assert_eq!(
        ast::scan_lines(Path::new("server.cjs"), "server.listen(0);\n"),
        vec![1]
    );
    assert_eq!(
        ast::scan_lines(Path::new("app.jsx"), "server.listen(0);\n"),
        vec![1]
    );
}

#[test]
fn listen_ts_wrappers_are_unwrapped() {
    assert_eq!(
        ast::scan_lines(Path::new("server.ts"), "server.listen(0 as number);\n"),
        vec![1]
    );
    assert_eq!(
        ast::scan_lines(
            Path::new("server.ts"),
            "server.listen(0 satisfies number);\n"
        ),
        vec![1]
    );
    assert_eq!(
        ast::scan_lines(Path::new("server.ts"), "server.listen((0));\n"),
        vec![1]
    );
    assert_eq!(
        ast::scan_lines(Path::new("server.ts"), "server.listen(0!);\n"),
        vec![1]
    );
    assert_eq!(
        ast::scan_lines(Path::new("server.ts"), "server.listen(<number>0);\n"),
        vec![1]
    );
    assert_eq!(
        ast::scan_lines(
            Path::new("server.ts"),
            "server.listen({ port: 0 as number });\n"
        ),
        vec![1]
    );
    assert_eq!(
        ast::scan_lines(
            Path::new("server.ts"),
            "server.listen({ ...opts, port: 0! });\n"
        ),
        vec![1]
    );
}

#[test]
fn empty_message_keeps_the_default() {
    let opts = compile_options(serde_yaml::from_str("message: \"\"\n").unwrap()).unwrap();
    assert_eq!(opts.message, DEFAULT_MESSAGE);
    let clone = Options::default();
    assert!(clone.include.is_empty());
    assert!(clone.allow.is_empty());
}

#[test]
fn default_include_covers_jsx() {
    let opts = compile_options(Options::default()).unwrap();
    assert!(opts.include.contains(&"**/*.jsx".to_string()));
}

#[test]
fn empty_file_list_is_silent() {
    let root = fixture("pass");
    let findings = check_with_files(&root, &config("{}"), &[]).unwrap();
    assert!(findings.is_empty());
}

#[test]
fn invalid_allow_glob_is_an_error() {
    let error = compile_options(serde_yaml::from_str("allow: [\"[\"]\n").unwrap())
        .err()
        .expect("invalid allow glob")
        .to_string();
    assert!(error.contains("no-raw-ephemeral-port allow"), "{error}");
}

#[test]
fn deferred_suppression_still_emits_disabled_file() {
    let findings = run_deferred(&fixture("disable"), "{}", true);
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].file, "server.ts");
}

#[test]
fn next_line_disable_is_skipped_unless_deferred() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("server.ts");
    std::fs::write(
        &path,
        "// no-mistakes-disable-next-line no-raw-ephemeral-port\nserver.listen(0);\n",
    )
    .unwrap();
    let sources = super::super::source_store_for_files(std::slice::from_ref(&path));
    let opts = compile_options(Options::default()).unwrap();
    assert!(scan::check_file(dir.path(), &path, &opts, &sources, false).is_empty());
    assert_eq!(
        scan::check_file(dir.path(), &path, &opts, &sources, true).len(),
        1
    );
}

#[test]
fn same_line_bind_and_listen_dedup() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("server.py");
    std::fs::write(&path, "s.bind((\"127.0.0.1\", 0)); s.bind((host, 0x0))\n").unwrap();
    let sources = super::super::source_store_for_files(std::slice::from_ref(&path));
    let opts = compile_options(Options::default()).unwrap();
    let findings = scan::check_file(dir.path(), &path, &opts, &sources, false);
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].line, 1);
}

#[test]
fn js_function_bind_is_silent() {
    let dir = tempfile::tempdir().unwrap();
    let opts = compile_options(Options::default()).unwrap();
    for name in ["handler.ts", "handler.js"] {
        let path = dir.path().join(name);
        std::fs::write(&path, "handler.bind((context, 0));\n").unwrap();
        let sources = super::super::source_store_for_files(std::slice::from_ref(&path));
        let findings = scan::check_file(dir.path(), &path, &opts, &sources, false);
        assert!(findings.is_empty(), "{name}: {findings:?}");
    }
    let path = dir.path().join("server.ts");
    std::fs::write(&path, "handler.bind((context, 0)); server.listen(0);\n").unwrap();
    let sources = super::super::source_store_for_files(std::slice::from_ref(&path));
    let findings = scan::check_file(dir.path(), &path, &opts, &sources, false);
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].line, 1);
}
