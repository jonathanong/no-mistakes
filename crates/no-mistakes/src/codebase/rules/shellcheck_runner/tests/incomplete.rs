use super::*;

fn selected() -> (PathBuf, Vec<PathBuf>) {
    let root = fixture_root("unavailable");
    let scripts = vec![
        root.join("scripts/check.sh"),
        root.join("scripts/next-disabled.sh"),
    ];
    (root, scripts)
}

#[test]
fn missing_executable_marks_each_selected_script_unanalyzable() {
    let (root, scripts) = selected();
    let result = handle_shellcheck_result(
        &root,
        &scripts,
        Err(std::io::Error::from(std::io::ErrorKind::NotFound)),
    )
    .unwrap();
    assert_eq!(result.len(), 2);
    assert!(result
        .iter()
        .all(|finding| finding.target.as_deref() == Some("unanalyzable") && finding.line == 1));
    assert!(result[0].message.contains("install shellcheck"));
}

#[test]
fn unsuccessful_execution_without_recognized_diagnostics_is_incomplete() {
    let (root, scripts) = selected();
    for code in [1, 2] {
        let result = handle_shellcheck_result(
            &root,
            &scripts,
            Ok(std::process::Output {
                status: make_exit_status(code),
                stdout: b"unrecognized tool output".to_vec(),
                stderr: b"execution failed".to_vec(),
            }),
        )
        .unwrap();
        assert_eq!(result.len(), 2);
        assert!(result
            .iter()
            .all(|finding| finding.target.as_deref() == Some("unanalyzable")));
    }
}

#[test]
fn partial_execution_preserves_definite_issues_and_marks_remaining_scripts_incomplete() {
    let (root, scripts) = selected();
    let result = handle_shellcheck_result(
        &root,
        &scripts,
        Ok(std::process::Output {
            status: make_exit_status(2),
            stdout: format!("{}:2:1: warning: issue [SC2086]\n", scripts[0].display()).into_bytes(),
            stderr: Vec::new(),
        }),
    )
    .unwrap();
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].target, None);
    assert_eq!(result[1].target.as_deref(), Some("unanalyzable"));
}

#[test]
fn handle_shellcheck_result_not_found_without_selected_scripts_returns_empty() {
    // No selected script means there is no incomplete analysis to report.
    let root = fixture_root("unavailable");
    let err = std::io::Error::new(std::io::ErrorKind::NotFound, "shellcheck not found");
    let result = handle_shellcheck_result(&root, &[], Err(err));
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}

#[test]
fn handle_shellcheck_result_other_error_propagates() {
    // Exercises the generic Err arm: any non-NotFound I/O error → Err.
    let root = fixture_root("unavailable");
    let err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied");
    let result = handle_shellcheck_result(&root, &[], Err(err));
    assert!(result.is_err());
}

#[test]
fn handle_shellcheck_result_success_returns_empty() {
    // Exercises the Ok(output) arm with exit code 0 → no findings.
    let root = fixture_root("unavailable");
    let output = std::process::Output {
        status: make_exit_status(0),
        stdout: Vec::new(),
        stderr: Vec::new(),
    };
    let result = handle_shellcheck_result(&root, &[], Ok(output));
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}

#[test]
fn handle_shellcheck_result_reports_only_affected_files() {
    // Only files mentioned in gcc-format stdout get findings; clean files are skipped.
    let (root, scripts) = selected();
    let sh1 = scripts[0].clone();
    let sh2 = scripts[1].clone();
    let gcc_line = format!("{}:2:1: warning: blah [SC2006]\n", sh1.display());
    let output = std::process::Output {
        status: make_exit_status(1),
        stdout: gcc_line.into_bytes(),
        stderr: Vec::new(),
    };
    let result = handle_shellcheck_result(&root, &[sh1.clone(), sh2], Ok(output));
    let findings = result.unwrap();
    assert_eq!(
        findings.len(),
        1,
        "only the flagged file should get a finding"
    );
    assert_eq!(findings[0].file, "scripts/check.sh");
}

#[test]
fn incomplete_policy_never_hides_definite_shellcheck_findings() {
    use crate::config::v2::Unanalyzable::{Advise, Ignore, Report};
    let (root, scripts) = selected();
    for policy in [Report, Advise, Ignore] {
        let findings = handle_shellcheck_result(
            &root,
            &scripts,
            Ok(std::process::Output {
                status: make_exit_status(2),
                stdout: format!("{}:2:1: warning: issue [SC2086]\n", scripts[0].display())
                    .into_bytes(),
                stderr: Vec::new(),
            }),
        )
        .unwrap();
        let diagnostics = super::super::diagnostics::route_findings(policy, None, findings);
        assert_eq!(
            diagnostics
                .findings
                .iter()
                .filter(|finding| finding.target.is_none())
                .count(),
            1
        );
        assert_eq!(
            diagnostics.findings.len(),
            if policy == Report { 2 } else { 1 }
        );
        assert_eq!(diagnostics.advisories.len(), usize::from(policy == Advise));
    }
}
