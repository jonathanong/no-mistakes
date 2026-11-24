use std::path::PathBuf;
use std::process::Command;

fn run_queues_check(root: &std::path::Path, extra: &[&str]) -> std::process::Output {
    let mut args = vec![
        "--lock-timeout",
        "0",
        "queues",
        "--root",
        root.to_str().unwrap(),
        "--format",
        "json",
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
    let bare = run_queues_check(&root, &[]);
    assert!(bare.status.success());
    assert_eq!(String::from_utf8_lossy(&bare.stdout).trim(), "[]");
    assert!(
        bare.stderr.is_empty(),
        "{}",
        String::from_utf8_lossy(&bare.stderr)
    );
    let full = run_queues_check(&root, &["--include-diagnostics"]);
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
    let bare = run_queues_check(&root, &[]);
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
