mod common;

use common::{fixture, run, stdout};

#[test]
fn flow_json_reports_symbol_neighborhood() {
    let root = fixture("tests-impact-symbol");
    let output = run(&[
        "flow",
        "utils.mts#parseDate",
        "--root",
        root.to_str().unwrap(),
        "--direction",
        "dependents",
        "--depth",
        "1",
        "--json",
    ]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(json["target"], "utils.mts#parseDate");
    assert!(json["nodes"].as_array().unwrap().iter().any(|node| {
        node["kind"] == "symbol" && node["file"] == "utils.mts" && node["symbol"] == "parseDate"
    }));
    assert!(
        !json["edges"].as_array().unwrap().is_empty(),
        "expected at least one symbol flow edge"
    );
}

#[test]
fn flow_supports_directions_relationships_and_formats() {
    let root = fixture("tests-impact-symbol");
    for (direction, format) in [
        ("deps", "paths"),
        ("both", "md"),
        ("dependents", "human"),
        ("deps", "yml"),
    ] {
        let output = run(&[
            "flow",
            "utils.mts#parseDate",
            "--root",
            root.to_str().unwrap(),
            "--direction",
            direction,
            "--depth",
            "2",
            "--relationship",
            "import",
            "--format",
            format,
        ]);

        assert!(
            output.status.success(),
            "{direction}/{format} stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            !stdout(&output).is_empty(),
            "{direction}/{format} should render output"
        );
    }
}

#[test]
fn flow_file_target_at_zero_depth_reports_only_target() {
    let root = fixture("tests-impact-symbol");
    let output = run(&[
        "flow",
        "utils.mts",
        "--root",
        root.to_str().unwrap(),
        "--depth",
        "0",
        "--json",
    ]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(json["target"], "utils.mts");
    assert_eq!(json["nodes"].as_array().unwrap().len(), 1);
    assert!(json["edges"].as_array().unwrap().is_empty());
}

#[test]
fn flow_import_target_keeps_vitest_config_indexable() {
    let root = no_mistakes::codebase::ts_resolver::normalize_path(
        &std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/test-plan/vitest-setup-dependencies"),
    );
    let output = run(&[
        "flow",
        "vitest.config.ts",
        "--root",
        root.to_str().unwrap(),
        "--direction",
        "deps",
        "--depth",
        "1",
        "--relationship",
        "import",
        "--json",
    ]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert!(
        json["edges"].as_array().unwrap().iter().any(|edge| {
            edge["from"] == "vitest.config.ts"
                && edge["to"] == "config/setup-selector.ts"
                && edge["kind"] == "import"
        }),
        "{json:#}"
    );
}
