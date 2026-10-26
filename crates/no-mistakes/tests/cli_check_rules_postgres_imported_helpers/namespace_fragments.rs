use super::*;

#[test]
fn namespace_builder_fragments_never_claim_clear_after_an_importer_mutates_them() {
    check_fragments("namespace-fragment-mutation", 2);
}

#[test]
fn named_builder_fragments_never_claim_clear_after_an_importer_mutates_them() {
    check_fragments("named-fragment-mutation", 4);
}

fn check_fragments(name: &str, builders: usize) {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-no-offset/variants")
        .join(name);
    for effort in ["low", "max"] {
        for policy in ["report", "advise", "ignore"] {
            let config = if policy == "report" {
                ".no-mistakes.yml".to_string()
            } else {
                format!("{policy}.yml")
            };
            let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
                .args([
                    "check",
                    "--format",
                    "json",
                    "--include-sql-verdicts",
                    "--root",
                ])
                .arg(&root)
                .arg("--config")
                .arg(root.join(config))
                .args(["--effort", effort])
                .output()
                .unwrap();
            let report: serde_json::Value =
                serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
                    panic!("{error}: {}", String::from_utf8_lossy(&output.stderr))
                });
            assert_eq!(
                output.status.code(),
                Some(i32::from(policy == "report")),
                "{effort}/{policy}: {report}"
            );
            let unknown = builders + usize::from(effort == "low");
            assert_eq!(
                report["rules"].as_array().unwrap().len(),
                if policy == "report" { unknown } else { 0 },
                "{effort}/{policy}: {report}"
            );
            assert_eq!(
                report["advisories"].as_array().unwrap().len(),
                if policy == "advise" { unknown } else { 0 },
                "{effort}/{policy}: {report}"
            );
            let verdicts = report["sqlVerdicts"].as_array().unwrap();
            assert_eq!(verdicts.len(), builders + 1, "{effort}/{policy}: {report}");
            for verdict in verdicts {
                // A const export fixes the binding, not the shared builder's contents.
                // Both mutated and apparently unmodified objects must abstain.
                let expected = if effort == "max" && verdict["file"] == "src/string.mts" {
                    "proven-clear"
                } else {
                    "abstain"
                };
                assert_eq!(verdict["status"], expected, "{effort}/{policy}: {report}");
            }
        }
    }
}
