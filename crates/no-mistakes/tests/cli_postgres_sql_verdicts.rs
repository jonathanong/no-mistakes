use std::path::PathBuf;
use std::process::Command;

fn fixture() -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "../../test-cases/rules/postgres-require-query-annotation/fixture/provenance-advice",
        ),
    )
}

fn report(include_verdicts: bool) -> serde_json::Value {
    let root = fixture();
    // Isolate the CLI invocation lock while keeping the saved SQL fixture intact.
    let runtime = tempfile::tempdir().unwrap();
    let mut command = Command::new(env!("CARGO_BIN_EXE_no-mistakes"));
    command.env("XDG_RUNTIME_DIR", runtime.path());
    command
        .args(["check", "--root"])
        .arg(&root)
        .arg("--config")
        .arg(root.join(".no-mistakes.yml"))
        .args(["--format", "json"]);
    if include_verdicts {
        command.arg("--include-sql-verdicts");
    }
    let output = command.output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

#[test]
fn sql_verdict_flag_only_adds_confirmed_call_evidence() {
    let plain = report(false);
    let mut detailed = report(true);
    assert!(plain.get("sqlVerdicts").is_none(), "{plain}");
    let rows = detailed["sqlVerdicts"].as_array().unwrap();
    assert_eq!(rows.len(), 5, "{detailed}");
    let source = std::fs::read_to_string(fixture().join("src/query.ts")).unwrap();
    let prefix_call = source
        .find("query(`/* known */ SELECT ${opaque}`)")
        .unwrap();
    assert!(
        rows.iter().any(|row| {
            row["file"] == "src/query.ts"
                && row["span"][0].as_u64() == Some(prefix_call as u64)
                && row["status"] == "proven-clear"
                && row["evidence"] == "verified-prefix"
                && row["knownPrefix"]
                    .as_str()
                    .is_some_and(|prefix| prefix.starts_with("/* known */ SELECT "))
        }),
        "{detailed}"
    );
    detailed.as_object_mut().unwrap().remove("sqlVerdicts");
    assert_eq!(detailed, plain);
}
