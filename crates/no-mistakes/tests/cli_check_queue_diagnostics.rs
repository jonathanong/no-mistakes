use std::path::PathBuf;
use std::process::Command;

fn run_queues_check(root: &std::path::Path, format: &str, extra: &[&str]) -> std::process::Output {
    let mut args = vec![
        "--lock-timeout",
        "0",
        "queues",
        "--root",
        root.to_str().unwrap(),
        "--format",
        format,
        "check",
    ];
    args.extend(extra);
    Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(args)
        .output()
        .unwrap()
}

#[test]
fn queues_check_ignores_non_queue_add_receivers() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/queue-ast-hop/no-queue-library-set-add/fixture")
        .canonicalize()
        .unwrap();
    let bare = run_queues_check(&root, "json", &[]);
    assert!(bare.status.success());
    assert_eq!(String::from_utf8_lossy(&bare.stdout).trim(), "[]");
    assert!(
        bare.stderr.is_empty(),
        "{}",
        String::from_utf8_lossy(&bare.stderr)
    );
    let full = run_queues_check(&root, "json", &["--include-diagnostics"]);
    assert!(full.status.success());
    let report: serde_json::Value = serde_json::from_slice(&full.stdout).unwrap();
    assert_eq!(report["findings"], serde_json::json!([]));
    assert_eq!(report["diagnostics"], serde_json::json!([]));
    assert!(
        full.stderr.is_empty(),
        "{}",
        String::from_utf8_lossy(&full.stderr)
    );
}

#[test]
fn queues_check_unresolved_producer_filter_snapshot() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/queue-ast-hop/unresolved-producer-filter/fixture")
        .canonicalize()
        .unwrap();
    let bare = run_queues_check(&root, "json", &[]);
    assert!(bare.status.success());
    assert_eq!(String::from_utf8_lossy(&bare.stdout).trim(), "[]");
    assert!(
        bare.stderr.is_empty(),
        "{}",
        String::from_utf8_lossy(&bare.stderr)
    );

    let mut reports = Vec::new();
    for jobs in ["1", "2"] {
        let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
            .args([
                "--lock-timeout",
                "0",
                "--jobs",
                jobs,
                "queues",
                "--root",
                root.to_str().unwrap(),
                "--format",
                "json",
                "check",
                "--include-diagnostics",
            ])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let stderr = String::from_utf8_lossy(&output.stderr);
        for expected in [
            "warning: imported-set.ts:4 dynamic or unresolved queue producer",
            "warning: param-receiver.ts:4 dynamic or unresolved queue producer",
            "warning: unknown-wrapper.ts:4 dynamic or unresolved queue producer",
        ] {
            assert!(stderr.contains(expected), "{stderr}");
        }
        assert!(!stderr.contains("local-builtins.ts"), "{stderr}");
        assert!(!stderr.contains("shared-set.ts"), "{stderr}");
        let mut parsed = serde_json::from_slice::<serde_json::Value>(&output.stdout).unwrap();
        parsed.sort_all_objects();
        reports.push(parsed);
    }
    assert_eq!(reports[0], reports[1]);
    insta::with_settings!({
        snapshot_path => "common/snapshots",
        prepend_module_to_snapshot => false,
        omit_expression => true,
    }, {
        insta::assert_snapshot!(
            "queues_check_unresolved_producer_filter",
            format!("{}\n", serde_json::to_string_pretty(&reports[0]).unwrap())
        );
    });
}

#[test]
fn queues_check_omits_empty_markdown_diagnostics_section() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/queue-ast-hop/no-queue-library-set-add/fixture");
    let output = run_queues_check(&root, "md", &["--include-diagnostics"]);
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "# Queue check"
    );
}

#[test]
fn queues_check_text_formats_include_opt_in_diagnostics_on_stderr() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/queue-ast-hop/unresolved-producer-filter/fixture");
    for format in ["paths", "human"] {
        let output = run_queues_check(&root, format, &["--include-diagnostics"]);
        assert!(output.status.success(), "{format}");
        assert!(output.stdout.is_empty(), "{format}");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("warning: imported-set.ts:4 dynamic or unresolved queue producer"),
            "{format}: {stderr}"
        );
    }
}

#[test]
fn queue_worker_rate_limit_suppression_check_json_snapshot() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/queue-ast-hop/worker-rate-limit-suppression/fixture")
        .canonicalize()
        .unwrap();
    let reports = crate::check_json_snapshot::assert_saved_fixture_report(
        "queue_worker_rate_limit_suppression",
        &[(
            "worker-rate-limit-suppression",
            root.clone(),
            root.join(".no-mistakes.yml"),
        )],
        &[],
        &["--include-suppressed"],
    );
    let result = &reports["worker-rate-limit-suppression"];
    assert_eq!(result["exitCode"], 0);
    let report = &result["report"];
    assert_eq!(report["rules"], serde_json::json!([]));
    assert_eq!(report["queues"], serde_json::json!([]));
    let warnings = report["warnings"].as_array().unwrap();
    assert_eq!(warnings.len(), 1, "{warnings:?}");
    assert!(warnings[0]
        .as_str()
        .unwrap()
        .starts_with("queues-check: worker.ts:8 "));
    assert_eq!(report["suppressed"].as_array().unwrap().len(), 3);

    let standalone = run_queues_check(&root, "human", &[]);
    assert!(standalone.status.success());
    let stderr = String::from_utf8_lossy(&standalone.stderr);
    for site in [
        "file-disabled.ts:7",
        "worker.ts:8",
        "worker.ts:10",
        "worker.ts:11",
    ] {
        assert!(stderr.contains(&format!("warning: {site} ")), "{stderr}");
    }
}
