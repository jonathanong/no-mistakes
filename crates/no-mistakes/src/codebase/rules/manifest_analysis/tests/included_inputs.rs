use super::*;

#[test]
fn selected_include_failures_are_routed_once_and_reads_are_shared() {
    let rule = pnpm_overrides_ban::RULE_ID;
    let root = fixture(rule, "unanalyzable-includes");
    let paths = [
        "broken.yaml",
        "invalid-utf8.yaml",
        "non-mapping.yaml",
        "missing.yaml",
    ];
    let mut files: Vec<_> = paths
        .iter()
        .map(|name| root.join("pnpm").join(name))
        .collect();
    files.push(root.join("pnpm/valid.yaml"));
    let sources = super::super::super::source_store_for_files(&files);
    for root_selected in [false, true] {
        if root_selected {
            files.push(root.join("pnpm-workspace.yaml"));
        }
        for (policy, expected_findings, expected_advisories) in
            [("report", 4, 0), ("advise", 0, 4), ("ignore", 0, 0)]
        {
            let diagnostics = pnpm_overrides_ban::diagnostics_with_files_and_sources(
                &root,
                &config(rule, &format!("unanalyzable: {policy}")),
                &files,
                &sources,
                false,
            )
            .unwrap();
            let selected = |finding: &&crate::codebase::rules::RuleFinding| {
                paths
                    .iter()
                    .any(|name| finding.file == format!("pnpm/{name}"))
            };
            assert_eq!(
                diagnostics.findings.iter().filter(selected).count(),
                expected_findings
            );
            assert_eq!(
                diagnostics.advisories.iter().filter(selected).count(),
                expected_advisories
            );
            for path in paths {
                let matches: Vec<_> = diagnostics
                    .findings
                    .iter()
                    .chain(&diagnostics.advisories)
                    .filter(|finding| finding.file == format!("pnpm/{path}"))
                    .collect();
                assert_eq!(matches.len(), usize::from(policy != "ignore"));
                assert!(matches
                    .iter()
                    .all(|finding| finding.target.as_deref() == Some("unanalyzable")));
            }
            assert!(diagnostics
                .findings
                .iter()
                .any(|finding| finding.file == "pnpm/valid.yaml"
                    && finding.target.as_deref() != Some("unanalyzable")));
            if root_selected {
                assert!(diagnostics
                    .findings
                    .iter()
                    .any(|finding| finding.message.contains("claimed by more than one")));
            }
            if root_selected && policy == "report" {
                assert!(diagnostics
                    .findings
                    .iter()
                    .any(|finding| finding.message.contains("invalid grant")));
            }
            // The root and every include share one request-owned SourceStore,
            // including failures and the unselected include.
            assert_eq!(sources.physical_read_count(), 7);
        }
    }
}
