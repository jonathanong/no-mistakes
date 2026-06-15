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

fn args(file: &str, names: &[&str]) -> DeadExportsArgs {
    DeadExportsArgs {
        file: PathBuf::from(file),
        names: names.iter().map(|name| name.to_string()).collect(),
        root: Some(fixture_root()),
        tsconfig: None,
        format: None,
        json: false,
    }
}

#[test]
fn detects_dead_export_across_all_exports() {
    let json = run_json(args("util.ts", &[])).unwrap();
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(value["anyDead"], true);
    let results = value["results"].as_array().unwrap();
    assert_eq!(results.len(), 3);
    let dead = results.iter().find(|r| r["name"] == "dead").unwrap();
    assert_eq!(dead["referenced"], false);
    assert_eq!(dead["importerCount"], 0);
}

#[test]
fn wildcard_and_default_imports_count_as_references() {
    // `used` is referenced only via a star re-export and a namespace import
    // (both indexed under `*`); the default export is referenced under
    // `default`. None should be reported dead.
    let report = compute(&DeadExportsArgs {
        file: PathBuf::from("mod.ts"),
        names: Vec::new(),
        root: Some(named_fixture("queries-reexport")),
        tsconfig: None,
        format: None,
        json: false,
    })
    .unwrap();
    assert!(!report.any_dead);
    let used = report.results.iter().find(|r| r.name == "used").unwrap();
    assert!(
        used.referenced,
        "used should be referenced via wildcard imports"
    );
    // The default export (declaration name `def`) is referenced under `default`.
    assert!(report
        .results
        .iter()
        .any(|r| r.referenced && r.name != "used"));
}

#[test]
fn star_barrel_does_not_keep_default_alive() {
    // `lonely`'s default is only seen by an `export *` barrel, which does not
    // forward defaults — so it is dead.
    let report = compute(&DeadExportsArgs {
        file: PathBuf::from("lonely.ts"),
        names: Vec::new(),
        root: Some(named_fixture("queries-reexport")),
        tsconfig: None,
        format: None,
        json: false,
    })
    .unwrap();
    assert!(report.any_dead);
}

#[test]
fn explicit_literal_default_counts_namespace_use() {
    // `dead-exports nsdef.ts default` must normalize to the default export, whose
    // only reference is a namespace import (`n.default()`).
    let report = compute(&DeadExportsArgs {
        file: PathBuf::from("nsdef.ts"),
        names: vec!["default".to_string()],
        root: Some(named_fixture("queries-reexport")),
        tsconfig: None,
        format: None,
        json: false,
    })
    .unwrap();
    assert!(report.results[0].referenced);
}

#[test]
fn explicit_default_display_name_is_normalized() {
    // Passing the declaration name `def` (as shown by exports-of) must still
    // resolve the default export's importers.
    let report = compute(&DeadExportsArgs {
        file: PathBuf::from("mod.ts"),
        names: vec!["def".to_string()],
        root: Some(named_fixture("queries-reexport")),
        tsconfig: None,
        format: None,
        json: false,
    })
    .unwrap();
    assert!(report.results[0].referenced);
}

#[test]
fn explicit_names_all_referenced() {
    let report = compute(&args("util.ts", &["used", "helper"])).unwrap();
    assert!(!report.any_dead);
    assert!(report.results.iter().all(|result| result.referenced));
}

#[test]
fn explicit_name_that_is_not_an_export_is_dead() {
    // `removed` is not (or no longer) an export of util.ts — the lookup falls
    // back to the raw name and finds no importers.
    let report = compute(&args("util.ts", &["removed"])).unwrap();
    assert!(report.any_dead);
    assert!(!report.results[0].referenced);
}

#[test]
fn deleted_name_ignores_wildcard_importers() {
    // `mod.ts` is consumed via `import * as m` and `export *` barrels, but a
    // deleted name still has no concrete import edge, so it is dead.
    let report = compute(&DeadExportsArgs {
        file: PathBuf::from("mod.ts"),
        names: vec!["removed".to_string()],
        root: Some(named_fixture("queries-reexport")),
        tsconfig: None,
        format: None,
        json: false,
    })
    .unwrap();
    assert!(report.any_dead);
    assert!(!report.results[0].referenced);
}

#[test]
fn renders_formats_runs_and_exit_codes() {
    let report = compute(&args("util.ts", &[])).unwrap();
    let mut human = Vec::new();
    render(&report, Format::Human, &mut human).unwrap();
    let text = String::from_utf8(human).unwrap();
    assert!(text.contains("dead: DEAD"));
    assert!(text.contains("used: referenced (3)"));

    let mut paths = Vec::new();
    render(&report, Format::Paths, &mut paths).unwrap();
    assert_eq!(String::from_utf8(paths).unwrap(), "dead\n");

    for format in [Format::Json, Format::Yml, Format::Md] {
        let mut buf = Vec::new();
        render(&report, format, &mut buf).unwrap();
        assert!(!buf.is_empty());
    }

    let _dead = run(args("util.ts", &[])).unwrap();
    let _alive = run(args("util.ts", &["used"])).unwrap();
    let _ = compute(&args("util.ts", &[])).unwrap().exit_code();
    let _ = compute(&args("util.ts", &["used"])).unwrap().exit_code();
}
