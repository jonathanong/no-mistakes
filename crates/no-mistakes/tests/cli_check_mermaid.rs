#[path = "common/saved_fixture.rs"]
mod saved_fixture;

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_no-mistakes"))
}

fn fixture() -> tempfile::TempDir {
    let fixture = saved_fixture::materialize("rules", "markdown-mermaid-validation");
    assert!(git(
        fixture.path(),
        &["init", "-q", "--initial-branch=main"]
    ));
    assert!(git(fixture.path(), &["add", "."]));
    fixture
}

fn git(root: &Path, args: &[&str]) -> bool {
    Command::new("git")
        .args(["-C", root.to_str().unwrap()])
        .args(args)
        .env_remove("GIT_DIR")
        .env_remove("GIT_COMMON_DIR")
        .env_remove("GIT_INDEX_FILE")
        .env_remove("GIT_WORK_TREE")
        .status()
        .unwrap()
        .success()
}

fn run(config: &str) -> Output {
    let fixture = fixture();
    let root = fixture.path();
    Command::new(bin())
        .args(["check", "--root"])
        .arg(root)
        .args([
            "--config",
            root.join(config).to_str().unwrap(),
            "--format",
            "json",
        ])
        .output()
        .unwrap()
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

#[test]
fn reports_invalid_and_unclosed_fences() {
    let output = run(".no-mistakes.yml");
    let body = stdout(&output);
    assert_eq!(
        output.status.code(),
        Some(1),
        "stdout: {body}\nstderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: serde_json::Value = serde_json::from_str(&body).unwrap_or_else(|error| {
        panic!(
            "expected JSON report: {error}; stdout: {body:?}; stderr: {:?}",
            String::from_utf8_lossy(&output.stderr)
        )
    });
    let findings = report["rules"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|finding| finding["rule"] == "markdown-mermaid-validation")
        .collect::<Vec<_>>();

    assert_eq!(findings.len(), 11, "{body}");
    for file in [
        "invalid-flowchart.md",
        "invalid-markdown.markdown",
        "invalid-mdx.mdx",
        "invalid-sequence.md",
        "invalid-state.md",
    ] {
        let finding = findings
            .iter()
            .find(|finding| finding["file"] == file)
            .unwrap_or_else(|| panic!("missing {file}: {body}"));
        assert_eq!(finding["line"], 3, "{finding:#?}");
        assert!(
            finding["message"]
                .as_str()
                .is_some_and(|message| message.contains("invalid Mermaid diagram")),
            "{finding:#?}"
        );
    }
    let multiple = findings
        .iter()
        .find(|finding| finding["file"] == "multiple.md")
        .unwrap_or_else(|| panic!("missing multiple.md: {body}"));
    assert_eq!(multiple["line"], 8, "{multiple:#?}");
    let jsx_adjacent = findings
        .iter()
        .find(|finding| finding["file"] == "jsx-adjacent-invalid.mdx")
        .unwrap_or_else(|| panic!("missing jsx-adjacent-invalid.mdx: {body}"));
    assert_eq!(jsx_adjacent["line"], 4, "{jsx_adjacent:#?}");
    assert!(jsx_adjacent["message"]
        .as_str()
        .is_some_and(|message| message.contains("invalid Mermaid diagram")));
    for file in [
        "unclosed.md",
        "unclosed-tab-indented.md",
        "unclosed-top-level-quoted-closer.md",
        "unclosed-blockquote-wrong-depth.md",
    ] {
        let unclosed = findings
            .iter()
            .find(|finding| finding["file"] == file)
            .unwrap_or_else(|| panic!("missing {file}: {body}"));
        assert_eq!(unclosed["line"], 3, "{unclosed:#?}");
        assert!(unclosed["message"]
            .as_str()
            .is_some_and(|message| message.contains("unclosed Mermaid fence")));
    }
    for ignored in ["excluded.md", "ignored.md", "suppressed.md", "valid.md"] {
        assert!(
            findings.iter().all(|finding| finding["file"] != ignored),
            "unexpected finding for {ignored}: {body}"
        );
    }
}

#[test]
fn validation_is_opt_in() {
    let output = run("no-rules.yml");
    let body = stdout(&output);
    assert!(
        output.status.success(),
        "stdout: {body}\nstderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(report["rules"], serde_json::json!([]), "{body}");
}
