use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn root(name: &str) -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!(
            "../../test-cases/rules/unconstructed-error-class/{name}/fixture"
        )),
    )
}

fn check(root: &Path, config: &str) -> (Output, serde_json::Value) {
    let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["check", "--format", "json", "--root"])
        .arg(root)
        .arg("--config")
        .arg(root.join(config))
        .output()
        .unwrap();
    let report = serde_json::from_slice(&output.stdout)
        .unwrap_or_else(|error| panic!("{error}: {}", String::from_utf8_lossy(&output.stderr)));
    (output, report)
}

fn targets(report: &serde_json::Value, channel: &str) -> Vec<String> {
    report[channel]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| {
            format!(
                "{}:{} {}",
                row["file"].as_str().unwrap(),
                row["line"],
                row["target"].as_str().unwrap()
            )
        })
        .collect()
}

#[test]
fn complete_package_constructor_proof_exits_zero_and_unused_errors_still_fail() {
    let root = root("shared-package-targets");
    let (output, report) = check(&root, "positive-only.yml");
    assert_eq!(output.status.code(), Some(0), "{report}");
    for channel in ["rules", "advisories", "warnings"] {
        assert_eq!(report[channel], serde_json::json!([]), "{report}");
    }
    for (config, advisories) in [
        (".no-mistakes.yml", 2),
        ("advise.yml", 2),
        ("ignore.yml", 0),
    ] {
        let (output, report) = check(&root, config);
        assert_eq!(output.status.code(), Some(1), "{report}");
        assert_eq!(
            targets(&report, "rules"),
            ["src/errors.ts:3 UnusedError", "src/errors.ts:5 UnusedTwin"],
            "{report}"
        );
        assert_eq!(report["advisories"].as_array().unwrap().len(), advisories);
        assert!(report["advisories"].as_array().unwrap().iter().all(|row| {
            matches!(
                row["file"].as_str(),
                Some("src/mutation-before.ts" | "src/opaque.ts")
            )
        }));
    }
}

#[test]
fn ambient_constructor_proof_preserves_unused_error_and_unknown_base_channels() {
    let (output, report) = check(&root("ambient-module-targets"), ".no-mistakes.yml");
    assert_eq!(output.status.code(), Some(1), "{report}");
    assert_eq!(
        targets(&report, "rules"),
        ["src/main.ts:3 UnusedError"],
        "{report}"
    );
    assert_eq!(
        targets(&report, "advisories"),
        [
            "src/opaque.ts:5 unanalyzable",
            "src/opaque.ts:6 unanalyzable",
            "src/opaque.ts:7 unanalyzable",
            "src/opaque.ts:8 unanalyzable",
        ],
        "{report}"
    );
}

#[test]
fn complete_declarations_remove_legacy_package_noise_in_cli() {
    let (output, report) = check(&root("real-imports"), ".no-mistakes.yml");
    assert_eq!(output.status.code(), Some(1), "{report}");
    assert_eq!(
        targets(&report, "rules"),
        ["src/errors.ts:1 UnusedError"],
        "{report}"
    );
    assert_eq!(report["advisories"], serde_json::json!([]), "{report}");
}

#[test]
fn workspace_export_proof_exits_zero_without_hiding_unused_app_error() {
    let root = root("workspace-export-targets");
    let (output, report) = check(&root, "positive-only.yml");
    assert_eq!(output.status.code(), Some(0), "{report}");
    assert_eq!(report["rules"], serde_json::json!([]), "{report}");
    assert_eq!(report["advisories"], serde_json::json!([]), "{report}");
    let (output, report) = check(&root, ".no-mistakes.yml");
    assert_eq!(output.status.code(), Some(1), "{report}");
    assert_eq!(
        targets(&report, "rules"),
        ["app/queues/errors.mts:3 UnusedError"],
        "{report}"
    );
    assert_eq!(report["advisories"], serde_json::json!([]), "{report}");
}
