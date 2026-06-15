use super::*;
use crate::cli::Format;
use crate::codebase::queries::render::render;
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

fn args(file: &str, no_importers: bool) -> ExportsOfArgs {
    ExportsOfArgs {
        file: PathBuf::from(file),
        root: Some(fixture_root()),
        tsconfig: None,
        no_importers,
        format: None,
        json: false,
    }
}

#[test]
fn lists_exports_with_importers() {
    let json = run_json(args("util.ts", false)).unwrap();
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();
    let exports = value["exports"].as_array().unwrap();
    assert_eq!(exports[0]["name"], "used");
    assert_eq!(
        exports[0]["importers"],
        serde_json::json!(["barrel.ts", "broken.ts", "consumer.ts"])
    );
    assert_eq!(exports[1]["name"], "dead");
    assert_eq!(exports[1]["importers"], serde_json::json!([]));
}

#[test]
fn no_importers_skips_reverse_scan() {
    let report = compute(&args("util.ts", true)).unwrap();
    assert!(report
        .exports
        .iter()
        .all(|export| export.importers.is_empty()));
}

#[test]
fn tags_every_export_kind() {
    let args = ExportsOfArgs {
        file: PathBuf::from("kinds.ts"),
        root: Some(named_fixture("queries-kinds")),
        tsconfig: None,
        no_importers: true,
        format: None,
        json: false,
    };
    let report = compute(&args).unwrap();
    let kinds: Vec<&str> = report.exports.iter().map(|export| export.kind).collect();
    for kind in [
        "function",
        "class",
        "const",
        "let",
        "var",
        "type",
        "interface",
        "enum",
        "default",
    ] {
        assert!(kinds.contains(&kind), "missing kind {kind}");
    }
}

#[test]
fn star_export_row_shows_concrete_importers() {
    // `export * from './mod'` consumers import concrete names from the barrel,
    // so the star export row's importers are those concrete consumers.
    let args = ExportsOfArgs {
        file: PathBuf::from("star-barrel.ts"),
        root: Some(named_fixture("queries-reexport")),
        tsconfig: None,
        no_importers: false,
        format: None,
        json: false,
    };
    let report = compute(&args).unwrap();
    let star = report
        .exports
        .iter()
        .find(|e| e.kind == "re-export")
        .unwrap();
    assert!(star.importers.contains(&"star-consumer.ts".to_string()));
}

#[test]
fn namespace_reexport_is_not_treated_as_star_row() {
    // `export * as api from './mod'` is a concrete export named `api`, so its
    // importers are only consumers of `api` — not every importer of the file.
    let args = ExportsOfArgs {
        file: PathBuf::from("ns-reexport.ts"),
        root: Some(named_fixture("queries-reexport")),
        tsconfig: None,
        no_importers: false,
        format: None,
        json: false,
    };
    let report = compute(&args).unwrap();
    let api = report.exports.iter().find(|e| e.name == "api").unwrap();
    assert_eq!(api.importers, vec!["api-consumer.ts".to_string()]);
}

#[test]
fn reexport_via_js_specifier_resolves_source() {
    // `export { x } from './dep.js'` resolves the re-export to its `.ts` source.
    let report = compute(&ExportsOfArgs {
        file: PathBuf::from("js-barrel.ts"),
        root: Some(named_fixture("queries-kinds")),
        tsconfig: None,
        no_importers: true,
        format: None,
        json: false,
    })
    .unwrap();
    assert_eq!(report.exports[0].resolved.as_deref(), Some("dep.ts"));
}

#[test]
fn reexport_resolves_target() {
    let report = compute(&args("barrel.ts", true)).unwrap();
    assert_eq!(report.exports[0].name, "used");
    assert_eq!(report.exports[0].kind, "re-export");
    assert_eq!(report.exports[0].resolved.as_deref(), Some("util.ts"));
}

#[test]
fn renders_formats_and_runs() {
    let report = compute(&args("util.ts", false)).unwrap();
    let mut human = Vec::new();
    render(&report, Format::Human, &mut human).unwrap();
    let text = String::from_utf8(human).unwrap();
    assert!(text.contains("used (function)"));
    assert!(text.contains("dead (const) line 7 <- (no importers)"));

    let mut paths = Vec::new();
    render(&report, Format::Paths, &mut paths).unwrap();
    let listed = String::from_utf8(paths).unwrap();
    assert!(listed.contains("consumer.ts"));

    for format in [Format::Json, Format::Yml, Format::Md] {
        let mut buf = Vec::new();
        render(&report, format, &mut buf).unwrap();
        assert!(!buf.is_empty());
    }
    let _ = run(args("util.ts", false)).unwrap();
}
