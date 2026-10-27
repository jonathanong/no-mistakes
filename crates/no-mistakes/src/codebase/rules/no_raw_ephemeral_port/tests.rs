use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::{Path, PathBuf};

// Output cases moved to cli_rra_snapshot_no_raw_ephemeral_port.

pub(super) fn fixture(name: &str) -> PathBuf {
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

pub(super) fn run_deferred(root: &Path, yaml: &str, defer_suppression: bool) -> Vec<RuleFinding> {
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
fn unreadable_file_reports_unanalyzable_input() {
    let root = fixture("pass");
    let path = root.join("missing.py");
    let sources = super::super::source_store_for_files(&[]);
    let opts = compile_options(Options::default()).unwrap();
    assert_eq!(
        scan::check_file(&root, &path, &opts, &sources, false)[0]
            .target
            .as_deref(),
        Some("unanalyzable")
    );
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
    let root = fixture("next-line-disable");
    let path = root.join("server.ts");
    let sources = super::super::source_store_for_files(std::slice::from_ref(&path));
    let opts = compile_options(Options::default()).unwrap();
    assert!(scan::check_file(&root, &path, &opts, &sources, false).is_empty());
    assert_eq!(
        scan::check_file(&root, &path, &opts, &sources, true).len(),
        1
    );
}

#[test]
fn prefilter_preserves_fixture_findings() {
    let opts = compile_options(Options::default()).unwrap();
    for name in ["fail-python", "fail-listen", "pass", "allow", "disable"] {
        let root = fixture(name);
        let files = crate::codebase::ts_source::discover_files(&root, &[]);
        let sources = super::super::source_store_for_files(&files);
        for path in &files {
            let filtered = scan::check_file(&root, path, &opts, &sources, false);
            let unfiltered =
                scan::check_file_with_prefilter(&root, path, &opts, &sources, false, false);
            assert_eq!(filtered, unfiltered, "{}", path.display());
        }
    }
}

#[test]
fn prefilter_skips_files_without_bind_or_listen() {
    scan::BIND_PREFILTER_SKIPS.with(|skips| skips.set(0));
    scan::LISTEN_PREFILTER_SKIPS.with(|skips| skips.set(0));
    let root = fixture("pass");
    let opts = compile_options(Options::default()).unwrap();
    let py = root.join("plain.py");
    let ts = root.join("plain.ts");
    let sources = super::super::source_store_for_files(&[py.clone(), ts.clone()]);
    assert!(scan::check_file(&root, &py, &opts, &sources, false).is_empty());
    assert!(scan::check_file(&root, &ts, &opts, &sources, false).is_empty());
    assert!(
        scan::BIND_PREFILTER_SKIPS.with(|skips| skips.get()) >= 1,
        "Python files without bind must skip the tuple-bind regex"
    );
    assert!(
        scan::LISTEN_PREFILTER_SKIPS.with(|skips| skips.get()) >= 1,
        "JS/TS files without listen or a backslash must skip the AST scan"
    );
}
