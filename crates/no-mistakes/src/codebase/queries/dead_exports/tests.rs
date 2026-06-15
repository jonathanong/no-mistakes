use super::*;
use crate::cli::Format;
use crate::codebase::queries::render::render;
use std::path::PathBuf;

fn fixture_root() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/queries/fixture"),
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
fn explicit_names_all_referenced() {
    let report = compute(&args("util.ts", &["used", "helper"])).unwrap();
    assert!(!report.any_dead);
    assert!(report.results.iter().all(|result| result.referenced));
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
