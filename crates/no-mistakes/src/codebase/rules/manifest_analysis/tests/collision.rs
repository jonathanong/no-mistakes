use super::*;

#[test]
fn package_named_unanalyzable_retains_definite_exemption_violations() {
    let rule = pnpm_release_age_policy::RULE_ID;
    let root = fixture(rule, "package-named-unanalyzable");
    let files = [
        root.join("pnpm-workspace.yaml"),
        root.join("package.json"),
        root.join("packages/broken/package.json"),
    ];
    let sources = super::super::super::source_store_for_files(&files);
    for (policy, uncertain, advice) in [("report", 1, 0), ("advise", 0, 1), ("ignore", 0, 0)] {
        for custom in [false, true] {
            for defer in [false, true] {
                let mut cfg = config(rule, &format!("unanalyzable: {policy}\npermanentPackages: [{{name: acme, reason: first-party}}]"));
                if custom {
                    cfg.rules[0].message = Some("Repair policy:".to_owned());
                }
                let diagnostics = pnpm_release_age_policy::diagnostics_with_files_and_sources(
                    &root, &cfg, &files, &sources, defer,
                )
                .unwrap();
                assert_eq!(
                    diagnostics.findings.len(),
                    2 + uncertain,
                    "{policy} custom={custom} defer={defer}"
                );
                assert_eq!(diagnostics.advisories.len(), advice);
                let definite: Vec<_> = diagnostics
                    .findings
                    .iter()
                    .filter(|finding| finding.file == "pnpm-workspace.yaml")
                    .collect();
                assert_eq!(definite.len(), 2);
                assert!(definite
                    .iter()
                    .all(|finding| finding.target.as_deref() == Some("unanalyzable")));
                assert!(definite
                    .iter()
                    .any(|finding| finding.message.contains("duplicates")));
                assert!(definite.iter().any(|finding| finding
                    .message
                    .contains("not in a release-age exemption registry")));
                if custom {
                    assert!(definite
                        .iter()
                        .all(|finding| finding.message.starts_with("Repair policy:")));
                }
                assert!(diagnostics
                    .advisories
                    .iter()
                    .all(|finding| finding.file == "packages/broken/package.json"));
            }
        }
    }
}
