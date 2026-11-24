use std::path::PathBuf;
use std::process::Command;

#[test]
fn queue_worker_rate_limit_check_json_snapshot() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/queue-ast-hop/worker-rate-limit/fixture");
    let reports = crate::check_json_snapshot::assert_saved_fixture_report(
        "queue_worker_rate_limit",
        &[(
            "worker-rate-limit",
            root.clone(),
            root.join(".no-mistakes.yml"),
        )],
        &[],
        &[],
    );
    let report = &reports["worker-rate-limit"]["report"];
    assert_eq!(reports["worker-rate-limit"]["exitCode"], 1);
    let findings = report["queues"].as_array().unwrap();
    assert!(findings
        .iter()
        .all(|finding| finding["kind"] == "worker-rate-limit-per-process"));
    assert!(findings.iter().all(|finding| {
        let file = finding["file"].as_str().unwrap_or("");
        file != "queue-level-limiter.ts"
            && file != "non-worker.ts"
            && file != "unresolved.ts"
            && file != "suppressed.ts"
    }));
    let warnings = report["warnings"].as_array().unwrap();
    assert_eq!(warnings.len(), 4, "{warnings:?}");
    assert!(warnings.iter().all(|warning| {
        let text = warning.as_str().unwrap_or("");
        text.starts_with("queues-check: unresolved.ts:")
            && text.contains("worker-rate-limit-per-process was not judged")
    }));
    for key in ["rules", "codebase", "integration", "react", "advisories"] {
        assert!(
            report[key].as_array().unwrap().is_empty(),
            "{key}: {report}"
        );
    }
}

#[test]
fn queue_worker_rate_limit_unresolved_surfaces_cli_diagnostics() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/queue-ast-hop/worker-rate-limit-unresolved/fixture")
        .canonicalize()
        .unwrap();
    let expected_message =
        "worker-rate-limit-per-process was not judged because the .rateLimit() receiver did not resolve";

    let aggregate = crate::check_json_snapshot::assert_saved_fixture_report(
        "queue_worker_rate_limit_unresolved",
        &[("worker-param", root.clone(), root.join(".no-mistakes.yml"))],
        &[],
        &[],
    );
    assert_eq!(aggregate["worker-param"]["exitCode"], 0);
    assert_eq!(
        aggregate["worker-param"]["report"]["queues"],
        serde_json::json!([])
    );
    assert_eq!(
        aggregate["worker-param"]["report"]["warnings"],
        serde_json::json!([format!(
            "queues-check: worker-param.ts:4 {expected_message}"
        )])
    );

    let bare_json = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args([
            "--lock-timeout",
            "0",
            "queues",
            "--root",
            root.to_str().unwrap(),
            "--format",
            "json",
            "check",
        ])
        .output()
        .unwrap();
    assert!(bare_json.status.success());
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&bare_json.stdout).unwrap(),
        serde_json::json!([])
    );
    let bare_stderr = String::from_utf8_lossy(&bare_json.stderr);
    assert!(
        bare_stderr.contains(&format!("warning: worker-param.ts:4 {expected_message}")),
        "{bare_stderr}"
    );

    let mut structured_reports = Vec::new();
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
        let mut parsed = serde_json::from_slice::<serde_json::Value>(&output.stdout).unwrap();
        parsed.sort_all_objects();
        structured_reports.push(parsed);
    }
    assert_eq!(structured_reports[0], structured_reports[1]);
    insta::with_settings!({
        snapshot_path => "common/snapshots",
        prepend_module_to_snapshot => false,
        omit_expression => true,
    }, {
        insta::assert_snapshot!(
            "queue_worker_rate_limit_unresolved_queues_check",
            format!("{}\n", serde_json::to_string_pretty(&structured_reports[0]).unwrap())
        );
    });

    let human = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args([
            "--lock-timeout",
            "0",
            "queues",
            "--root",
            root.to_str().unwrap(),
            "check",
        ])
        .output()
        .unwrap();
    assert!(human.status.success());
    assert!(String::from_utf8_lossy(&human.stdout).trim().is_empty());
    let human_stderr = String::from_utf8_lossy(&human.stderr);
    assert!(
        human_stderr.contains(&format!("warning: worker-param.ts:4 {expected_message}")),
        "{human_stderr}"
    );

    let md = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args([
            "--lock-timeout",
            "0",
            "queues",
            "--root",
            root.to_str().unwrap(),
            "--format",
            "md",
            "check",
            "--include-diagnostics",
        ])
        .output()
        .unwrap();
    assert!(md.status.success());
    let md_stdout = String::from_utf8_lossy(&md.stdout);
    assert!(md_stdout.contains("## diagnostics"), "{md_stdout}");
    assert!(md_stdout.contains(expected_message), "{md_stdout}");

    let yml = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args([
            "--lock-timeout",
            "0",
            "queues",
            "--root",
            root.to_str().unwrap(),
            "--format",
            "yml",
            "check",
            "--include-diagnostics",
        ])
        .output()
        .unwrap();
    assert!(yml.status.success());
    let yml_stdout = String::from_utf8_lossy(&yml.stdout);
    assert!(yml_stdout.contains("diagnostics:"), "{yml_stdout}");
    assert!(yml_stdout.contains(expected_message), "{yml_stdout}");
}
