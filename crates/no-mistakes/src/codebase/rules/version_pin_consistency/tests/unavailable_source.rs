use super::*;

fn saved_config(name: &str) -> NoMistakesConfig {
    let root = fixture("unavailable-source");
    crate::config::v2::load_v2_config(&root, Some(&root.join(name))).unwrap()
}

#[test]
fn unavailable_declared_source_routes_standalone_findings() {
    let root = fixture("unavailable-source");
    for prefix in ["", "missing-"] {
        for (policy, count) in [("report", 1), ("advise", 0), ("ignore", 0)] {
            let cfg = saved_config(&format!("{prefix}{policy}.yml"));
            let findings = run_config(&root, &cfg, &["README.md"]);
            assert_eq!(findings.len(), count, "{prefix}{policy}: {findings:?}");
            if count > 0 {
                assert_eq!(findings[0].file, "README.md");
                assert_eq!(findings[0].line, 1);
                assert_eq!(findings[0].target.as_deref(), Some("unanalyzable"));
                assert!(findings[0].message.contains("Restore"));
            }
        }
    }
}

#[test]
fn incomplete_source_diagnostics_keep_both_channels_and_selected_scope() {
    let root = fixture("unavailable-source");
    let files = vec![root.join("README.md")];
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let diagnostics = diagnostics_with_files_and_sources(
        &root,
        &saved_config("advise.yml"),
        &files,
        &sources,
        false,
    )
    .unwrap();
    assert!(diagnostics.findings.is_empty());
    assert_eq!(diagnostics.advisories.len(), 1);
    assert!(diagnostics.advisories[0].message.contains("versions.toml"));
    assert!(diagnostics.advisories[0]
        .message
        .contains("correct its syntax"));
    let mut cfg = saved_config("report.yml");
    cfg.rules[0].include = vec!["unselected.md".into()];
    assert!(run_config(&root, &cfg, &["README.md"]).is_empty());
}

#[test]
fn valid_unselected_source_and_definite_mismatches_survive_every_policy() {
    let root = fixture("unavailable-source");
    for policy in ["report", "advise", "ignore"] {
        let cfg = saved_config(&format!("mixed-{policy}.yml"));
        let findings = run_config(&root, &cfg, &["README.md"]);
        assert_eq!(findings.len(), if policy == "report" { 2 } else { 1 });
        assert_eq!(
            findings
                .iter()
                .filter(|f| f.message.contains("version mismatch"))
                .count(),
            1
        );
    }
}

#[test]
fn cached_read_error_is_not_an_empty_source_or_a_second_read() {
    let root = fixture("unavailable-source");
    let mut cfg = saved_config("report.yml");
    cfg.rules[0].options["sourceFile"] = "invalid-utf8.toml".into();
    let files = vec![root.join("README.md")];
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let first = diagnostics_with_files_and_sources(&root, &cfg, &files, &sources, true).unwrap();
    let second = diagnostics_with_files_and_sources(&root, &cfg, &files, &sources, true).unwrap();
    assert_eq!(first.findings, second.findings);
    assert_eq!(first.findings.len(), 1);
    assert!(first.findings[0].message.contains("cannot read"));
    assert_eq!(sources.physical_read_count(), 1);
}

#[test]
fn anchor_suppressions_apply_to_both_channels_and_defer_for_audit() {
    let root = fixture("unavailable-source");
    for name in ["disabled.md", "line.md"] {
        for policy in ["report", "advise"] {
            let mut cfg = saved_config(&format!("{policy}.yml"));
            cfg.rules[0].include = vec![name.into()];
            cfg.rules[0].options["anchors"][0]["file"] = name.into();
            let files = vec![root.join(name)];
            let sources = crate::codebase::rules::source_store_for_files(&files);
            let result =
                diagnostics_with_files_and_sources(&root, &cfg, &files, &sources, false).unwrap();
            assert!(result.findings.is_empty());
            assert!(result.advisories.is_empty());
            let deferred =
                diagnostics_with_files_and_sources(&root, &cfg, &files, &sources, true).unwrap();
            assert_eq!(deferred.findings.len() + deferred.advisories.len(), 1);
        }
    }
}

#[test]
fn selected_source_validation_and_empty_source_are_not_uncertainty() {
    let root = fixture("unavailable-source");
    for source in ["versions.toml", "empty.toml"] {
        for policy in ["report", "advise", "ignore"] {
            let mut cfg = saved_config(&format!("{policy}.yml"));
            cfg.rules[0].include.clear();
            cfg.rules[0].options["sourceFile"] = source.into();
            let findings = run_config(&root, &cfg, &[source, "README.md"]);
            assert_eq!(findings.len(), 1);
            assert_eq!(findings[0].file, source);
            assert_eq!(findings[0].target, None);
            assert!(findings[0].message.contains(if source == "empty.toml" {
                "not found"
            } else {
                "failed to parse TOML"
            }));
        }
    }
}

#[test]
fn failed_dependency_reports_only_selected_anchors() {
    let root = fixture("unavailable-source");
    let mut cfg = saved_config("report.yml");
    cfg.rules[0].include.clear();
    let extra = cfg.rules[0].options["anchors"][0].clone();
    cfg.rules[0].options["anchors"]
        .as_sequence_mut()
        .unwrap()
        .push(extra);
    cfg.rules[0].options["anchors"][1]["file"] = "unselected.md".into();
    let findings = run_config(&root, &cfg, &["README.md"]);
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, "README.md");
}

#[test]
fn selected_unreadable_source_still_reports_without_selected_anchors() {
    let root = fixture("unavailable-source");
    for policy in ["report", "advise", "ignore"] {
        let mut cfg = saved_config(&format!("{policy}.yml"));
        cfg.rules[0].include.clear();
        cfg.rules[0].options["sourceFile"] = "invalid-utf8.toml".into();
        // The configured anchor is not selected; source validation must survive.
        let findings = run_config(&root, &cfg, &["invalid-utf8.toml"]);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].file, "invalid-utf8.toml");
        assert_eq!(findings[0].target, None);
        assert!(findings[0].message.contains("cannot read"));
    }
}

#[test]
fn unreadable_selected_anchor_routes_actual_error_through_every_policy() {
    let root = fixture("unavailable-source");
    let files = vec![root.join("invalid-utf8.md")];
    let sources = crate::codebase::rules::source_store_for_files(&files);
    for policy in ["report", "advise", "ignore"] {
        let mut cfg = saved_config(&format!("{policy}.yml"));
        cfg.rules[0].include = vec!["invalid-utf8.md".into()];
        cfg.rules[0].options["sourceFile"] = "valid.toml".into();
        cfg.rules[0].options["anchors"][0]["file"] = "invalid-utf8.md".into();
        let first =
            diagnostics_with_files_and_sources(&root, &cfg, &files, &sources, false).unwrap();
        let second =
            diagnostics_with_files_and_sources(&root, &cfg, &files, &sources, false).unwrap();
        assert_eq!(first.findings, second.findings, "{policy}");
        assert_eq!(first.advisories, second.advisories, "{policy}");
        let expected_findings = usize::from(policy == "report");
        let expected_advisories = usize::from(policy == "advise");
        assert_eq!(first.findings.len(), expected_findings, "{policy}");
        assert_eq!(first.advisories.len(), expected_advisories, "{policy}");
        let finding = first.findings.first().or_else(|| first.advisories.first());
        if let Some(finding) = finding {
            assert_eq!(finding.file, "invalid-utf8.md");
            assert_eq!(finding.line, 1);
            assert_eq!(finding.target.as_deref(), Some("unanalyzable"));
            assert!(finding.message.contains("cannot read invalid-utf8.md"));
            assert!(finding.message.contains("invalid utf-8 sequence"));
            assert!(finding.message.contains("readable UTF-8 text"));
            assert!(!finding.message.contains("could not find"));
        }
    }
    // The readable source and failed anchor are each read once across all checks.
    assert_eq!(sources.physical_read_count(), 2);
}
