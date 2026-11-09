use serde_json::Value;
use std::path::PathBuf;
use std::process::Command;

const RULE: &str = "test-no-live-repository-scan";

fn root(name: &str) -> PathBuf {
    crate::check_json_snapshot::rules_path(RULE, "caller-roots", name)
}

fn report(name: &str) -> Value {
    let root = root(name);
    let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["check", "--format", "json", "--root"])
        .arg(&root)
        .arg("--config")
        .arg(root.join(".no-mistakes.yml"))
        .output()
        .unwrap();
    serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
        panic!(
            "invalid check JSON ({error}): {}",
            String::from_utf8_lossy(&output.stderr)
        )
    })
}

fn fixture_line(name: &str, file: &str, marker: &str) -> usize {
    let source = std::fs::read_to_string(root(name).join(file)).unwrap();
    source
        .lines()
        .position(|line| line.contains(marker))
        .unwrap_or_else(|| panic!("missing {marker} in {name}/{file}"))
        + 1
}

fn locations(report: &Value) -> Vec<String> {
    let mut found = Vec::new();
    for field in ["rules", "advisories"] {
        for item in report[field].as_array().into_iter().flatten() {
            if item["rule"] == RULE {
                found.push(format!(
                    "{}:{}",
                    item["file"].as_str().unwrap(),
                    item["line"]
                ));
            }
        }
    }
    found.sort();
    found
}

#[test]
fn caller_proven_roots_match_small_and_large_projects() {
    let cases: Vec<(&str, PathBuf, PathBuf)> = [
        "minimal",
        "production-only",
        "large-unknown",
        "large-collision",
        "small",
        "large",
        "no-caller",
        "disabled",
        "unknown-setup",
    ]
    .into_iter()
    .map(|name| {
        let root = root(name);
        (name, root.clone(), root.join(".no-mistakes.yml"))
    })
    .collect();
    crate::check_json_snapshot::assert_saved_fixture(
        "test_no_live_repository_scan_caller_roots",
        &cases,
    );
}

#[test]
fn owned_callers_clear_without_clearing_controls() {
    for name in [
        "minimal",
        "production-only",
        "large-unknown",
        "large-collision",
    ] {
        let found = locations(&report(name));
        assert!(
            !found.contains(&"src/pb.mts:5".to_string()),
            "{name}: {found:?}"
        );
    }

    for name in ["no-caller", "disabled"] {
        let found = locations(&report(name));
        assert!(found.is_empty(), "{name}: {found:?}");
    }

    assert_eq!(locations(&report("unknown-setup")), ["src/pb.mts:5"]);

    let found = locations(&report("large"));
    for owned in [
        "src/scan.ts:5",
        "src/scan.ts:9",
        "src/scan.ts:13",
        "src/scan.ts:17",
        "src/long-chain.ts:131",
    ] {
        assert!(
            !found.contains(&owned.to_string()),
            "owned {owned}: {found:?}"
        );
    }
    let spread_callback = format!(
        "tests/scan.test.ts:{}",
        fixture_line("large", "tests/scan.test.ts", "// raw-spread-forwarder")
    );
    assert!(
        !found.contains(&spread_callback),
        "raw spread callback {spread_callback}: {found:?}"
    );

    for control in [
        "src/scan.ts:21",
        "src/scan.ts:25",
        "src/scan.ts:29",
        "src/scan.ts:33",
        "src/long-chain.ts:260",
    ] {
        assert!(
            found.contains(&control.to_string()),
            "control {control}: {found:?}"
        );
    }
}
