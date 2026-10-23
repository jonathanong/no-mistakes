use super::super::{diagnostics_with_files_and_sources, RuleFinding};
use super::effort::run_prepared;
use super::{config, fixture, fixture_files, run};

fn missing(findings: &[RuleFinding]) -> Vec<RuleFinding> {
    findings
        .iter()
        .filter(|finding| finding.message.contains("has no matching App Router page"))
        .cloned()
        .collect()
}

#[test]
fn delegated_low_preserves_exact_max_missing_findings_and_routes_unknowns() {
    for fixture in ["low-reexport", "low-path-alias"] {
        let baseline = run(fixture, "{}");
        let expected = missing(&baseline);
        assert_eq!(expected.len(), 2, "{baseline:?}");
        assert!(expected
            .iter()
            .all(|finding| finding.file == "next-shared.ts"));
        for policy in ["report", "advise", "ignore"] {
            let (findings, advisories) = run_prepared(
                fixture,
                &format!("{{ effort: low, unanalyzable: {policy} }}"),
            );
            assert_eq!(missing(&findings), expected, "{fixture} {policy}");
            let unknown: Vec<_> = findings
                .iter()
                .filter(|finding| finding.message.contains("unanalyzable"))
                .collect();
            assert_eq!(unknown.len(), if policy == "report" { 2 } else { 0 });
            assert_eq!(advisories.len(), if policy == "advise" { 2 } else { 0 });
            for finding in unknown.into_iter().chain(advisories.iter()) {
                assert_eq!(finding.file, "next-shared.ts");
                assert!(finding.message.contains("effort low"));
                assert!(finding.message.contains("return a literal array"));
                assert_eq!(
                    finding.line,
                    if finding.message.contains("rewrites()") {
                        9
                    } else {
                        3
                    }
                );
            }
            assert_eq!(
                run(
                    fixture,
                    &format!("{{ effort: max, unanalyzable: {policy} }}")
                ),
                baseline
            );
        }
    }
}

#[test]
fn delegated_file_and_line_suppressions_survive_both_diagnostic_channels() {
    for name in ["low-delegate-disabled", "low-delegate-suppressed"] {
        for policy in ["report", "advise", "ignore"] {
            let root = fixture(name);
            let files = fixture_files(&root);
            let sources = crate::codebase::rules::source_store_for_files(&files);
            let mut config = config(&format!("{{ effort: low, unanalyzable: {policy} }}"));
            crate::codebase::rules::analysis_policy::prepare_rule_analysis(
                &root,
                &mut config,
                &files,
                &sources,
            )
            .unwrap();
            let diagnostics =
                diagnostics_with_files_and_sources(&root, &config, &files, &sources, None, false)
                    .unwrap();
            let (findings, advisories) = (diagnostics.findings, diagnostics.advisories);
            assert!(findings.is_empty(), "{name}: {findings:?}");
            assert!(advisories.is_empty(), "{name}: {advisories:?}");
        }
    }
}

#[test]
fn combined_diagnostics_matches_standalone_channels_and_keeps_configured_message_behavior() {
    for name in ["low-reexport", "low-path-alias"] {
        let root = fixture(name);
        let files = fixture_files(&root);
        let sources = crate::codebase::rules::source_store_for_files(&files);
        let mut config = config("{ effort: low, unanalyzable: advise }");
        config.rules[0].message = Some("preserve configured guidance".into());
        crate::codebase::rules::analysis_policy::prepare_rule_analysis(
            &root,
            &mut config,
            &files,
            &sources,
        )
        .unwrap();
        let combined =
            diagnostics_with_files_and_sources(&root, &config, &files, &sources, None, false)
                .unwrap();
        let (findings, advisories) = run_prepared(name, "{ effort: low, unanalyzable: advise }");
        assert_eq!(combined.findings, findings);
        assert_eq!(combined.advisories, advisories);
    }
}

#[test]
fn unresolved_local_delegates_report_incomplete_extraction_without_base_url_fallback() {
    for (fixture, specifier, line) in [
        ("limit-alias-precedence", "next-shared", 1),
        ("limit-missing-exact-alias", "next-shared", 1),
        ("limit-missing-wildcard-alias", "next-shared", 1),
        ("fail-unresolved-relative", "./missing-config", 2),
        ("fail-unresolved-alias", "@config", 2),
    ] {
        let baseline = run(fixture, "{}");
        assert_eq!(baseline.len(), 1, "{fixture}: {baseline:?}");
        let finding = &baseline[0];
        assert_eq!(finding.file, "next.config.ts");
        assert_eq!(finding.line, line);
        assert!(finding.message.contains("extraction is incomplete"));
        assert!(finding
            .message
            .contains(&format!("specifier '{specifier}'")));
        assert!(finding.message.contains("restore the local module"));
        for policy in ["report", "advise", "ignore"] {
            let (findings, advisories) = run_prepared(
                fixture,
                &format!("{{ effort: low, unanalyzable: {policy} }}"),
            );
            assert_eq!(
                findings,
                if policy == "report" {
                    baseline.clone()
                } else {
                    vec![]
                }
            );
            assert_eq!(
                advisories,
                if policy == "advise" {
                    baseline.clone()
                } else {
                    vec![]
                }
            );
            let (findings, advisories) = run_prepared(
                fixture,
                &format!("{{ effort: max, unanalyzable: {policy} }}"),
            );
            assert_eq!(findings, baseline);
            assert!(advisories.is_empty());
        }
    }
}

#[test]
fn unresolved_delegates_respect_entry_config_suppressions_in_both_channels() {
    for name in [
        "unresolved-disabled",
        "unresolved-next-line",
        "unresolved-line",
    ] {
        for effort in ["low", "max"] {
            for policy in ["report", "advise", "ignore"] {
                let root = fixture(name);
                let files = fixture_files(&root);
                let sources = crate::codebase::rules::source_store_for_files(&files);
                let mut config = config(&format!("{{ effort: {effort}, unanalyzable: {policy} }}"));
                crate::codebase::rules::analysis_policy::prepare_rule_analysis(
                    &root,
                    &mut config,
                    &files,
                    &sources,
                )
                .unwrap();
                let diagnostics = diagnostics_with_files_and_sources(
                    &root, &config, &files, &sources, None, false,
                )
                .unwrap();
                assert!(
                    diagnostics.findings.is_empty(),
                    "{name}: {:?}",
                    diagnostics.findings
                );
                assert!(
                    diagnostics.advisories.is_empty(),
                    "{name}: {:?}",
                    diagnostics.advisories
                );
            }
        }
    }
}
