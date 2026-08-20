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
    assert!(scan::check_file(&root, &path, &opts, &sources).is_empty());
}

#[test]
fn bind_regex_accepts_whitespace_and_single_quotes() {
    assert_eq!(bind_lines("sock.bind(('127.0.0.1', 0))\n"), vec![1]);
    assert_eq!(bind_lines(". bind ( ( \"x\" , 0 ) )\n"), vec![1]);
    assert!(bind_lines("sock.bind((\"127.0.0.1\", 8080))\n").is_empty());
    assert!(bind_lines("sock.bind(0)\n").is_empty());
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
}

#[test]
fn listen_non_literal_ports_are_ignored() {
    assert!(ast::scan_lines(Path::new("server.ts"), "server.listen(8080);\n").is_empty());
    assert!(ast::scan_lines(Path::new("server.ts"), "server.listen({ port: 8080 });\n").is_empty());
    assert!(ast::scan_lines(Path::new("server.ts"), "server.listen(port);\n").is_empty());
    assert!(ast::scan_lines(Path::new("server.ts"), "server.listen({ port });\n").is_empty());
    assert!(ast::scan_lines(Path::new("server.ts"), "server.listen({ port: '0' });\n").is_empty());
    assert!(ast::scan_lines(Path::new("server.ts"), "listen(0);\n").is_empty());
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
}

#[test]
fn empty_message_keeps_the_default() {
    let opts = compile_options(serde_yaml::from_str("message: \"\"\n").unwrap()).unwrap();
    assert_eq!(opts.message, DEFAULT_MESSAGE);
    let clone = Options::default();
    assert!(clone.include.is_empty());
    assert!(clone.allow.is_empty());
}
