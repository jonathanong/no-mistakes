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

fn target_shape(report: &ExportsOfReport) -> Vec<(String, &'static str, u32, Option<String>)> {
    report
        .exports
        .iter()
        .map(|export| {
            (
                export.name.clone(),
                export.kind,
                export.line,
                export.resolved.clone(),
            )
        })
        .collect()
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
fn recovered_default_export_matches_no_importers_output() {
    let fixture = crate::codebase::queries::test_support::materialize_root_fixture(
        "recovered-target-symbols",
    );
    let root = crate::codebase::ts_resolver::normalize_path(fixture.path());

    let without_importers = compute(&ExportsOfArgs {
        file: PathBuf::from("target.ts"),
        root: Some(root.clone()),
        tsconfig: None,
        no_importers: true,
        format: None,
        json: false,
    })
    .unwrap();
    let with_importers = compute(&ExportsOfArgs {
        file: PathBuf::from("target.ts"),
        root: Some(root),
        tsconfig: None,
        no_importers: false,
        format: None,
        json: false,
    })
    .unwrap();

    assert_eq!(without_importers.exports.len(), 1);
    assert_eq!(without_importers.exports[0].kind, "default");
    assert_eq!(
        with_importers.exports[0].name,
        without_importers.exports[0].name
    );
    assert_eq!(
        with_importers.exports[0].kind,
        without_importers.exports[0].kind
    );
    assert_eq!(with_importers.exports[0].importers, vec!["consumer.ts"]);
}

#[test]
fn reverse_target_symbols_keep_typescript_parsing_for_js_and_mjs() {
    // `extract_symbols_at_path` has always parsed these extension-bearing
    // files as TypeScript. The project-wide reverse facts use their native JS
    // mode, so this parity check keeps the target-facing contract explicit.
    let root = named_fixture("symbols-output");
    for file in [
        "src/types-in-js.js",
        "src/types-in-mjs.mjs",
        // `.mts` uses a module source type in reverse facts but legacy target
        // extraction uses `SourceType::ts`, so it must not reuse those facts.
        "src/types.mts",
    ] {
        let without_importers = compute(&ExportsOfArgs {
            file: PathBuf::from(file),
            root: Some(root.clone()),
            tsconfig: None,
            no_importers: true,
            format: None,
            json: false,
        })
        .unwrap();
        let with_importers = compute(&ExportsOfArgs {
            file: PathBuf::from(file),
            root: Some(root.clone()),
            tsconfig: None,
            no_importers: false,
            format: None,
            json: false,
        })
        .unwrap();

        assert_eq!(
            target_shape(&with_importers),
            target_shape(&without_importers)
        );
        assert!(
            !with_importers.exports.is_empty(),
            "{file} must retain its TypeScript declarations"
        );
    }
}

#[test]
fn reverse_target_symbols_keep_legacy_parse_mode_for_cts_and_declarations() {
    let fixture =
        crate::codebase::queries::test_support::materialize_root_fixture("legacy-target-symbols");
    let root = crate::codebase::ts_resolver::normalize_path(fixture.path());
    for file in ["commonjs.cts", "declaration.d.ts"] {
        let without_importers = compute(&ExportsOfArgs {
            file: PathBuf::from(file),
            root: Some(root.clone()),
            tsconfig: None,
            no_importers: true,
            format: None,
            json: false,
        })
        .unwrap();
        let with_importers = compute(&ExportsOfArgs {
            file: PathBuf::from(file),
            root: Some(root.clone()),
            tsconfig: None,
            no_importers: false,
            format: None,
            json: false,
        })
        .unwrap();

        assert_eq!(
            target_shape(&with_importers),
            target_shape(&without_importers)
        );
        assert!(
            !with_importers.exports.is_empty(),
            "{file} must retain its TypeScript declarations"
        );
    }
}

#[test]
fn reverse_reexport_resolution_uses_the_target_nested_tsconfig() {
    let fixture =
        crate::codebase::queries::test_support::materialize_root_fixture("nested-target-alias");
    let root = crate::codebase::ts_resolver::normalize_path(fixture.path());
    let make_args = |no_importers| ExportsOfArgs {
        file: PathBuf::from("nested/src/barrel.ts"),
        root: Some(root.clone()),
        tsconfig: None,
        no_importers,
        format: None,
        json: false,
    };

    let without_importers = compute(&make_args(true)).unwrap();
    let with_importers = compute(&make_args(false)).unwrap();
    assert_eq!(
        without_importers.exports[0].resolved.as_deref(),
        Some("nested/src/value.ts")
    );
    assert_eq!(
        with_importers.exports[0].resolved,
        without_importers.exports[0].resolved
    );
}

#[test]
fn fatal_parse_error_is_not_recovered_as_an_export() {
    let fixture =
        crate::codebase::queries::test_support::materialize_root_fixture("fatal-target-symbols");
    let root = crate::codebase::ts_resolver::normalize_path(fixture.path());

    for no_importers in [true, false] {
        let error = match compute(&ExportsOfArgs {
            file: PathBuf::from("target.ts"),
            root: Some(root.clone()),
            tsconfig: None,
            no_importers,
            format: None,
            json: false,
        }) {
            Err(error) => format!("{error:#}"),
            Ok(_) => panic!("fatal parser failure must not yield exports"),
        };
        assert!(error.contains("extracting symbols from"), "{error}");
        if no_importers {
            // Direct extraction rejects OXC parser panics with its stable
            // symbol-extraction error. Reverse facts preserve the diagnostic
            // but mark it fatal for the same outcome.
            assert!(
                error.contains("failed to parse TypeScript source"),
                "{error}"
            );
        }
    }
}

#[test]
fn pass4b_exports_of_skips_ignored_reexport_for_visible_fallback() {
    let fixture = crate::test_support::materialize_gitignore_fixture("pass4b-shadow");
    crate::test_support::git_init(fixture.path());
    crate::test_support::git_add_all(fixture.path());
    let root = crate::codebase::ts_resolver::normalize_path(fixture.path());
    let report = compute(&ExportsOfArgs {
        file: PathBuf::from("query/source.ts"),
        root: Some(root),
        tsconfig: None,
        no_importers: true,
        format: None,
        json: false,
    })
    .unwrap();

    assert!(report
        .exports
        .iter()
        .all(|export| export.resolved.as_deref() == Some("query/target.ts")));
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
