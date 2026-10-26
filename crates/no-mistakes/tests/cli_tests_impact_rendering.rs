use std::path::PathBuf;
use std::process::{Command, Output};

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_no-mistakes"))
}

fn plan_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/tests-impact-rendering")
        .join(name)
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
fn tests_why_preserves_vitest_setup_provenance_from_plan_json() {
    let plan_file = plan_path("why-vitest-setup.json");

    let output = run(&[
        "tests",
        "why",
        "a.test.mts",
        "--plan",
        plan_file.to_str().unwrap(),
        "--format",
        "json",
    ]);

    assert!(output.status.success());
    let why: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(
        why["setup/helpers.mts"][1]["detail"]["type"],
        "vitest-setup"
    );
    assert_eq!(why["setup/helpers.mts"][1]["detail"]["field"], "setupFiles");
    assert!(why["setup/helpers.mts"][0]["detail"].is_null());
}

#[test]
fn tests_comment_formats_markdown() {
    let plan_file = plan_path("comment-markdown.json");

    let output = run(&["tests", "comment", plan_file.to_str().unwrap()]);

    assert!(output.status.success());
    let markdown = stdout(&output);
    assert!(markdown.contains("# 🧪 Test Impact Analysis"));
    assert!(markdown.contains("a.test.mts"));
    assert!(markdown.contains("🟢 High"));
}

#[test]
fn tests_graph_mermaid_outputs_flowchart() {
    let plan_file = plan_path("graph-mermaid.json");

    let output_mermaid = run(&[
        "tests",
        "graph",
        plan_file.to_str().unwrap(),
        "--format",
        "mermaid",
    ]);
    assert!(output_mermaid.status.success());
    let mermaid = stdout(&output_mermaid);
    assert!(mermaid.contains("graph TD"));
    assert!(mermaid.contains("classDef changed"));
    assert!(mermaid.contains("classDef test"));
    assert!(mermaid.contains("c.mts"));
    assert!(mermaid.contains("a.test.mts"));
    assert!(mermaid.contains("vitest-setup (setupFiles)"));

    let output_json = run(&[
        "tests",
        "graph",
        plan_file.to_str().unwrap(),
        "--format",
        "json",
    ]);
    assert!(output_json.status.success());
    let graph: serde_json::Value = serde_json::from_str(&stdout(&output_json)).unwrap();
    assert!(graph["nodes"].as_array().unwrap().len() >= 4);
    let edges = graph["edges"].as_array().unwrap();
    assert!(edges.iter().any(|edge| edge["via"] == "Import"));
    assert!(edges.iter().any(|edge| {
        edge["via"] == "vitest-setup"
            && edge["detail"]["type"] == "vitest-setup"
            && edge["detail"]["field"] == "setupFiles"
    }));
}
