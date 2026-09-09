use serde_json::Value;
use std::path::PathBuf;
use std::process::Command;

fn fixture() -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/rules/swift-csharp-lexical-masking"),
    )
}

#[test]
fn cli_reports_only_executable_swift_and_csharp_matches() {
    let root = fixture();
    let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["check", "--root"])
        .arg(&root)
        .args(["--config"])
        .arg(root.join(".no-mistakes.yml"))
        .args(["--format", "json"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    let findings = report["rules"].as_array().unwrap();
    assert_eq!(findings.len(), 10, "{report:#?}");
    assert!(findings.iter().any(|finding| {
        finding["rule"] == "swift-viewmodel-main-actor"
            && finding["file"] == "SwiftViewModel.swift"
            && finding["line"] == 13
    }));
    assert!(!findings
        .iter()
        .any(|finding| { finding["file"] == "SwiftPrint.swift" && finding["line"] == 14 }));
    assert!(!findings
        .iter()
        .any(|finding| { finding["file"] == "AsyncDelegate.cs" && finding["line"] == 13 }));
    assert!(findings.iter().any(|finding| {
        finding["rule"] == "csharp-no-async-void-delegate"
            && finding["file"] == "AsyncDelegate.cs"
            && finding["line"] == 18
    }));
    assert!(findings.iter().any(|finding| {
        finding["rule"] == "csharp-no-async-void-delegate"
            && finding["file"] == "AsyncDelegate.cs"
            && finding["line"] == 24
    }));
    assert!(findings.iter().any(|finding| {
        finding["rule"] == "swift-no-raw-print"
            && finding["file"] == "SwiftPrint.swift"
            && finding["line"] == 17
    }));
    assert!(!findings.iter().any(|finding| {
        finding["rule"] == "swift-no-raw-print"
            && finding["file"] == "SwiftPrint.swift"
            && finding["line"] == 22
    }));
}
