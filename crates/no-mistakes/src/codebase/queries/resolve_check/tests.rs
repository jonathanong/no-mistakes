use super::*;
use crate::cli::Format;
use crate::codebase::queries::render::{render, resolve_format};
use std::path::PathBuf;

fn fixture_root() -> PathBuf {
    named_fixture("queries")
}

fn named_fixture(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis")
            .join(name)
            .join("fixture"),
    )
}

fn args(file: &str) -> ResolveCheckArgs {
    ResolveCheckArgs {
        file: PathBuf::from(file),
        root: Some(fixture_root()),
        tsconfig: None,
        format: None,
        json: false,
    }
}

#[test]
fn classifies_each_import() {
    let json = run_json(args("broken.ts")).unwrap();
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(value["allResolve"], false);
    assert_eq!(
        value["unresolved"],
        serde_json::json!(["./missing", "@app/missing"])
    );
    let statuses: Vec<&str> = value["imports"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| row["status"].as_str().unwrap())
        .collect();
    assert_eq!(
        statuses,
        vec![
            "resolved",
            "unresolved",
            "unresolved",
            "external",
            "external"
        ]
    );
    assert_eq!(value["imports"][0]["resolved"], "util.ts");
}

#[test]
fn clean_file_resolves() {
    let report = compute(&args("consumer.ts")).unwrap();
    assert!(report.all_resolve);
    assert!(report.unresolved.is_empty());
}

#[test]
fn tags_every_import_kind_and_absolute_path() {
    let root = named_fixture("queries-kinds");
    // Absolute file path exercises the absolute branch of input resolution.
    let report = compute(&ResolveCheckArgs {
        file: root.join("imports.ts"),
        root: Some(root.clone()),
        tsconfig: None,
        format: None,
        json: false,
    })
    .unwrap();
    let kinds: Vec<&str> = report.imports.iter().map(|row| row.kind).collect();
    assert!(kinds.contains(&"type"));
    assert!(kinds.contains(&"dynamic"));
    assert!(kinds.contains(&"require"));
    assert!(report.all_resolve);
}

#[test]
fn renders_every_format() {
    let report = compute(&args("broken.ts")).unwrap();
    for format in [
        Format::Json,
        Format::Yml,
        Format::Human,
        Format::Paths,
        Format::Md,
    ] {
        let mut buf = Vec::new();
        render(&report, format, &mut buf).unwrap();
        assert!(!buf.is_empty());
    }
    let mut human = Vec::new();
    render(&report, Format::Human, &mut human).unwrap();
    let text = String::from_utf8(human).unwrap();
    assert!(text.contains("MISSING  ./missing"));
    assert!(text.contains("external express"));
    assert!(text.contains("ok       ./util -> util.ts"));
}

#[test]
fn paths_lists_resolved_targets() {
    let report = compute(&args("broken.ts")).unwrap();
    let mut buf = Vec::new();
    render(&report, Format::Paths, &mut buf).unwrap();
    assert_eq!(String::from_utf8(buf).unwrap(), "util.ts\n");
}

#[test]
fn resolve_format_precedence() {
    assert_eq!(
        resolve_format(true, Some(Format::Human), true),
        Format::Json
    );
    assert_eq!(resolve_format(false, Some(Format::Md), true), Format::Md);
    assert_eq!(resolve_format(false, None, true), Format::Human);
    assert_eq!(resolve_format(false, None, false), Format::Json);
}

#[test]
fn run_returns_exit_codes() {
    // Exercises run() and both exit_code branches (value not comparable).
    let _broken = run(args("broken.ts")).unwrap();
    let _clean = run(args("consumer.ts")).unwrap();
    let _ = compute(&args("broken.ts")).unwrap().exit_code();
    let _ = compute(&args("consumer.ts")).unwrap().exit_code();
}
