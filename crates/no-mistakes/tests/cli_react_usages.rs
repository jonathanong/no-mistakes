use std::path::PathBuf;
use std::process::{Command, Output};

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_no-mistakes"))
}

fn react_fixture(name: &str) -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/react-traits-usages")
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

#[test]
fn react_usages_json_reports_callsites_props_and_sections() {
    let root = react_fixture("basic");
    let output = run(&[
        "react",
        "--root",
        root.to_str().unwrap(),
        "--json",
        "usages",
        "app/components/button.tsx#Button",
    ]);
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(json["target"]["symbol"], "Button");
    assert_eq!(json["callsites"].as_array().unwrap().len(), 5);
    let spread = json["callsites"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["file"] == "app/pages/dashboard.tsx")
        .unwrap();
    assert_eq!(spread["hasSpread"], true);
    assert_eq!(json["stories"][0], "app/components/button.stories.tsx");
    assert_eq!(json["tests"][0], "app/components/button.test.tsx");
    assert_eq!(json["propTypes"][0], "ButtonProps");
}

#[test]
fn react_usages_missing_target_file_errors() {
    let root = react_fixture("basic");
    let output = run(&[
        "react",
        "--root",
        root.to_str().unwrap(),
        "usages",
        "app/components/missing.tsx",
    ]);
    assert!(!output.status.success());
}
