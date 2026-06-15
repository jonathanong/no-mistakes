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

fn args(file: &str, tests: bool) -> ImportersArgs {
    ImportersArgs {
        file: PathBuf::from(file),
        tests,
        root: Some(fixture_root()),
        tsconfig: None,
        format: None,
        json: false,
    }
}

#[test]
fn lists_direct_importers_and_count() {
    let json = run_json(args("util.ts", false)).unwrap();
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(
        value["directImporters"],
        serde_json::json!(["barrel.ts", "broken.ts", "consumer.ts"])
    );
    assert_eq!(value["dependentsCount"], 3);
    assert!(value.get("testImpact").is_none());
}

#[test]
fn tests_flag_adds_impacted_tests() {
    let report = compute(&args("util.ts", true)).unwrap();
    let impact = report.test_impact.expect("test impact present");
    assert_eq!(impact.count, 1);
    assert_eq!(impact.tests, vec!["consumer.test.ts".to_string()]);
}

#[test]
fn renders_formats_and_runs() {
    let report = compute(&args("util.ts", true)).unwrap();
    let mut human = Vec::new();
    render(&report, Format::Human, &mut human).unwrap();
    let text = String::from_utf8(human).unwrap();
    assert!(text.contains("util.ts (3 dependents)"));
    assert!(text.contains("impacts 1 tests:"));
    assert!(text.contains("consumer.test.ts"));

    let mut paths = Vec::new();
    render(&report, Format::Paths, &mut paths).unwrap();
    assert!(String::from_utf8(paths).unwrap().contains("barrel.ts"));

    for format in [Format::Json, Format::Yml, Format::Md] {
        let mut buf = Vec::new();
        render(&report, format, &mut buf).unwrap();
        assert!(!buf.is_empty());
    }

    // Human render without --tests: the test-impact section is omitted.
    let no_tests = compute(&args("util.ts", false)).unwrap();
    let mut human = Vec::new();
    render(&no_tests, Format::Human, &mut human).unwrap();
    let text = String::from_utf8(human).unwrap();
    assert!(!text.contains("impacts"));

    let _ = run(args("util.ts", false)).unwrap();
}
