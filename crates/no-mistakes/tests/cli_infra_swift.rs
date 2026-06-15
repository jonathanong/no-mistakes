use std::path::{Path, PathBuf};
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

fn run_in(root: &Path, args: &[&str]) -> Output {
    Command::new(bin())
        .args(args)
        .arg("--root")
        .arg(root.to_str().unwrap())
        .output()
        .expect("no-mistakes should run")
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("stdout should be utf8")
}

// --- infra ---

#[test]
fn infra_resource_refs_json_lists_referencing_blocks() {
    let root = fixture("terraform-basic");
    let output = run_in(
        &root,
        &["infra", "resource-refs", "aws_route53_record.foo", "--json"],
    );
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    let addresses: Vec<&str> = json
        .as_array()
        .unwrap()
        .iter()
        .map(|row| row["address"].as_str().unwrap())
        .collect();
    assert!(addresses.contains(&"aws_lb.web"));
    assert!(addresses.contains(&"output.record_id"));
}

#[test]
fn infra_resource_refs_renders_all_formats() {
    let root = fixture("terraform-basic");
    for format in ["yml", "md", "paths", "human"] {
        let output = run_in(
            &root,
            &[
                "infra",
                "resource-refs",
                "aws_route53_record.foo",
                "--format",
                format,
            ],
        );
        assert!(output.status.success(), "format {format} failed");
        assert!(stdout(&output).contains("main.tf"), "format {format}");
    }
}

#[test]
fn infra_outputs_reports_exports_and_consumers() {
    let root = fixture("terraform-basic");
    for format in ["json", "yml", "md", "paths", "human"] {
        let output = run_in(
            &root,
            &[
                "infra",
                "outputs",
                "infra/modules/network",
                "--format",
                format,
            ],
        );
        assert!(output.status.success(), "format {format} failed");
    }
    let output = run_in(
        &root,
        &["infra", "outputs", "infra/modules/network", "--json"],
    );
    let json: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(json["exports"][0]["name"], "zone_id");
    assert_eq!(json["consumers"][0]["output"], "zone_id");
}

#[test]
fn infra_test_for_finds_covering_test() {
    let root = fixture("terraform-basic");
    for format in ["json", "yml", "md", "paths", "human"] {
        let output = run_in(
            &root,
            &[
                "infra",
                "test-for",
                "infra/envs/prod/main.tf",
                "--format",
                format,
            ],
        );
        assert!(output.status.success(), "format {format} failed");
    }
    let output = run_in(
        &root,
        &[
            "infra",
            "test-for",
            "infra/envs/prod/main.tf",
            "--format",
            "paths",
        ],
    );
    assert!(stdout(&output).contains("network.test.mts"));
}

// --- swift ---

#[test]
fn swift_importers_lists_importing_files() {
    let root = fixture("swift-test-plan");
    let file = "swift-clients/core/Sources/VouchaAPI/Endpoint.swift";
    for format in ["yml", "md", "paths", "human"] {
        let output = run_in(&root, &["swift", "importers", file, "--format", format]);
        assert!(output.status.success(), "format {format} failed");
    }
    let output = run_in(&root, &["swift", "importers", file, "--json"]);
    let json: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    let files: Vec<&str> = json
        .as_array()
        .unwrap()
        .iter()
        .map(|row| row["file"].as_str().unwrap())
        .collect();
    assert!(files.iter().any(|f| f.ends_with("APIClient.swift")));
}

#[test]
fn swift_test_targets_reports_covering_target() {
    let root = fixture("swift-test-plan");
    let file = "swift-clients/core/Sources/VouchaAPI/Endpoint.swift";
    for format in ["yml", "md", "paths", "human"] {
        let output = run_in(&root, &["swift", "test-targets", file, "--format", format]);
        assert!(output.status.success(), "format {format} failed");
    }
    let output = run_in(&root, &["swift", "test-targets", file, "--json"]);
    let json: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    let targets: Vec<&str> = json
        .as_array()
        .unwrap()
        .iter()
        .map(|row| row["target"].as_str().unwrap())
        .collect();
    assert!(targets.contains(&"VouchaCoreTests"));
}
