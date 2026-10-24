use super::*;

#[test]
fn cached_invalid_utf8_accounts_directives_without_rereading_or_recovering_source() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "../../test-cases/rules/github-actions-script-expressions/fixture/unreadable-input",
        ),
    );
    let rule = "github-actions-script-expressions";
    for (name, line, kind, directive_line) in [
        ("file", 1, SuppressionDirectiveKind::File, 1),
        ("line", 1, SuppressionDirectiveKind::Line, 1),
        ("next", 2, SuppressionDirectiveKind::NextLine, 1),
    ] {
        let file = format!(".github/workflows/suppressed-{name}.yml");
        let path = root.join(&file);
        let sources = crate::codebase::rules::source_store_for_files(std::slice::from_ref(&path));
        let original_error = sources.read_path(&path).unwrap_err();
        assert_eq!(sources.physical_read_count(), 1);

        // The saved invalid bytes must remain an error for every analyzer, even
        // after suppression accounts for the deferred unreadable-input finding.
        for source_file in [file.clone(), path.to_string_lossy().into_owned()] {
            let mut findings = vec![rule, "unmatched-rule"];
            let suppressed = suppress_domain_findings_with_source_locations(
                &root,
                &mut findings,
                &sources,
                |rule| SuppressionTarget {
                    domain: "codebase",
                    rule,
                    file: "public-target.yml",
                    line: Some(7),
                    reason: "cannot read GitHub Actions source",
                    identity: None,
                },
                |_| Some((&source_file, Some(line))),
            );
            assert_eq!(findings, vec!["unmatched-rule"]);
            assert_eq!(
                suppressed,
                vec![SuppressedFinding {
                    domain: "codebase".to_string(),
                    rule: rule.to_string(),
                    file: "public-target.yml".to_string(),
                    source_file,
                    line: Some(7),
                    reason: "cannot read GitHub Actions source".to_string(),
                    directive: SuppressionDirective {
                        kind,
                        line: directive_line,
                    },
                }],
            );
            assert!(std::sync::Arc::ptr_eq(
                &original_error,
                &sources.read_path(&path).unwrap_err(),
            ));
            assert_eq!(sources.physical_read_count(), 1);
        }
    }
}
