use std::path::PathBuf;
use std::process::{Command, Output};

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_no-mistakes"))
}

fn fixture(name: &str) -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis")
            .join(name)
            .join("fixture"),
    )
}

fn run(args: &[&str]) -> Output {
    Command::new(bin())
        .args(args)
        .output()
        .expect("no-mistakes should run")
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("stdout should be utf8")
}

fn json(output: &Output) -> serde_json::Value {
    serde_json::from_str(&stdout(output)).expect("stdout should be json")
}

#[test]
fn importers_reports_direct_importers_and_count() {
    let root = fixture("queries");
    let output = run(&[
        "importers",
        "util.ts",
        "--root",
        root.to_str().unwrap(),
        "--json",
    ]);
    assert!(output.status.success());
    let value = json(&output);
    assert_eq!(value["dependentsCount"], 3);
    assert_eq!(
        value["directImporters"],
        serde_json::json!(["barrel.ts", "broken.ts", "consumer.ts"])
    );
}

#[test]
fn importers_tests_flag_adds_impacted_tests() {
    let root = fixture("queries");
    let output = run(&[
        "importers",
        "util.ts",
        "--root",
        root.to_str().unwrap(),
        "--tests",
        "--json",
    ]);
    assert!(output.status.success());
    assert_eq!(json(&output)["testImpact"]["count"], 1);
}

#[test]
fn exports_of_lists_exports_and_importers() {
    let root = fixture("queries");
    let output = run(&[
        "exports-of",
        "util.ts",
        "--root",
        root.to_str().unwrap(),
        "--json",
    ]);
    assert!(output.status.success());
    assert_eq!(json(&output)["exports"][0]["name"], "used");
}

#[test]
fn dead_exports_exits_non_zero_when_dead() {
    let root = fixture("queries");
    let output = run(&[
        "dead-exports",
        "util.ts",
        "--root",
        root.to_str().unwrap(),
        "--json",
    ]);
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(json(&output)["anyDead"], true);
}

#[test]
fn call_sites_reports_argument_shapes() {
    let root = fixture("queries");
    let output = run(&[
        "call-sites",
        "util.ts",
        "used",
        "--root",
        root.to_str().unwrap(),
        "--json",
    ]);
    assert!(output.status.success());
    assert_eq!(json(&output)["callSites"].as_array().unwrap().len(), 4);
}

#[test]
fn resolve_check_exit_codes() {
    let root = fixture("queries");
    let broken = run(&[
        "resolve-check",
        "broken.ts",
        "--root",
        root.to_str().unwrap(),
        "--json",
    ]);
    assert_eq!(broken.status.code(), Some(1));
    assert_eq!(json(&broken)["allResolve"], false);

    let clean = run(&[
        "resolve-check",
        "consumer.ts",
        "--root",
        root.to_str().unwrap(),
        "--json",
    ]);
    assert!(clean.status.success());
}

#[test]
fn human_output_is_readable() {
    let root = fixture("queries");
    let output = run(&[
        "dead-exports",
        "util.ts",
        "--root",
        root.to_str().unwrap(),
        "--format",
        "human",
    ]);
    assert!(stdout(&output).contains("dead: DEAD"));
}
