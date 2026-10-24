use super::super::{check_with_files, diagnostics_with_files_and_sources};
use crate::codebase::rules::source_store_for_files;
use crate::config::v2::NoMistakesConfig;
use std::path::PathBuf;

fn root() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "../../test-cases/rules/github-actions-script-expressions/fixture/unreadable-input",
        ),
    )
}

#[test]
fn unreadable_selected_sources_route_policy_without_reclassifying_expressions() {
    let root = root();
    let files: Vec<_> = [
        ".github/workflows/bad.yml",
        ".github/workflows/collision.yml",
        ".github/actions/local/action.yml",
        "unrelated.yml",
        ".github/workflows/suppressed-file.yml",
        ".github/workflows/suppressed-line.yml",
        ".github/workflows/suppressed-next.yml",
    ]
    .iter()
    .map(|file| root.join(file))
    .collect();
    for prefix in [
        "",
        "collision-",
        "mixed-",
        "composite-",
        "suppressed-",
        "unrelated-",
    ] {
        for (policy, failures, advice) in [("report", 1, 0), ("advise", 0, 1), ("ignore", 0, 0)] {
            let config: NoMistakesConfig = serde_yaml::from_str(
                &std::fs::read_to_string(root.join(format!("{prefix}{policy}.yml"))).unwrap(),
            )
            .unwrap();
            let sources = source_store_for_files(&files);
            let expected = match prefix {
                "collision-" => (1, 0),
                "mixed-" => (failures + 1, advice),
                "suppressed-" | "unrelated-" => (0, 0),
                _ => (failures, advice),
            };
            for deferred in [false, true] {
                let result =
                    diagnostics_with_files_and_sources(&root, &config, &files, &sources, deferred)
                        .unwrap();
                assert_eq!(
                    (result.findings.len(), result.advisories.len()),
                    expected,
                    "{prefix}{policy} deferred={deferred}"
                );
            }
            assert_eq!(
                check_with_files(&root, &config, &files).unwrap().len(),
                expected.0
            );
            // Standalone discovery must agree with the prepared-inventory entry point.
            assert_eq!(
                super::super::check(&root, &config).unwrap().len(),
                expected.0,
                "standalone {prefix}{policy}",
            );
            // Repeated diagnostics reuse failures as well as successful source reads.
            assert_eq!(
                sources.physical_read_count(),
                if prefix == "mixed-" {
                    2
                } else if prefix == "suppressed-" {
                    3
                } else if prefix == "unrelated-" {
                    0
                } else {
                    1
                }
            );
        }
    }
}

#[test]
fn invalid_utf8_failure_retains_bytes_for_directives_without_another_read() {
    let path = root().join(".github/workflows/suppressed-file.yml");
    let sources = source_store_for_files(std::slice::from_ref(&path));
    let first = sources.read_path(&path).unwrap_err();
    let second = sources.read_path(&path).unwrap_err();
    assert!(std::sync::Arc::ptr_eq(&first, &second));
    let retained = first
        .get_ref()
        .unwrap()
        .downcast_ref::<std::string::FromUtf8Error>()
        .unwrap();
    assert!(retained
        .as_bytes()
        .starts_with(b"# no-mistakes-disable-file"));
    assert_eq!(sources.physical_read_count(), 1);
}

#[test]
fn selected_missing_workflow_reports_io_failure_at_first_line() {
    let root = root();
    let config: NoMistakesConfig =
        serde_yaml::from_str(&std::fs::read_to_string(root.join("report.yml")).unwrap()).unwrap();
    let missing_root = root.join("missing-directory");
    let files = vec![missing_root.join(".github/workflows/bad.yml")];
    let sources = source_store_for_files(&files);
    let result =
        diagnostics_with_files_and_sources(&missing_root, &config, &files, &sources, false)
            .unwrap();
    assert_eq!(result.findings.len(), 1);
    assert_eq!(result.findings[0].line, 1);
    assert!(result.findings[0]
        .message
        .contains("cannot read GitHub Actions source"));
}
