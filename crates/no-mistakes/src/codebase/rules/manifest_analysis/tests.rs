use super::*;
use crate::codebase::rules::{
    package_json_registry_only, pnpm_overrides_ban, pnpm_release_age_policy,
};
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::PathBuf;

type Analyze =
    fn(&Path, &NoMistakesConfig, &[PathBuf], &SourceStore, bool) -> Result<AnalysisDiagnostics>;

fn fixture(rule: &str, scenario: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules")
            .join(rule)
            .join("fixture")
            .join(scenario),
    )
}

fn config(rule: &str, options: &str) -> NoMistakesConfig {
    NoMistakesConfig {
        rules: vec![RuleDef {
            rule: rule.to_owned(),
            scope: Some(RuleScope::Repository),
            options: serde_yaml::from_str(options).unwrap(),
            ..Default::default()
        }],
        ..Default::default()
    }
}

#[test]
fn malformed_manifests_route_policy_and_keep_definite_findings() {
    let cases: [(&str, Analyze); 3] = [
        (
            package_json_registry_only::RULE_ID,
            package_json_registry_only::diagnostics_with_files_and_sources,
        ),
        (
            pnpm_overrides_ban::RULE_ID,
            pnpm_overrides_ban::diagnostics_with_files_and_sources,
        ),
        (
            pnpm_release_age_policy::RULE_ID,
            pnpm_release_age_policy::diagnostics_with_files_and_sources,
        ),
    ];
    for (rule, analyze) in cases {
        let root = fixture(rule, "unanalyzable");
        let mut files: Vec<_> = [
            "broken",
            "invalid-utf8",
            "suppressed",
            "suppressed-utf8",
            "valid",
        ]
        .into_iter()
        .map(|name| root.join("packages").join(name).join("package.json"))
        .collect();
        if rule == pnpm_release_age_policy::RULE_ID {
            files.push(root.join("pnpm-workspace.yaml"));
        }
        let sources = super::super::source_store_for_files(&files);
        for (policy, failures, advisories) in [("report", 3, 0), ("advise", 1, 2), ("ignore", 1, 0)]
        {
            let options = format!(
                "unanalyzable: {policy}\n{}",
                if rule == pnpm_release_age_policy::RULE_ID {
                    "permanentPackages: [{name: acme, reason: first-party}]\n"
                } else {
                    ""
                }
            );
            let mut cfg = config(rule, &options);
            cfg.rules[0].message = Some("Repair metadata:".to_owned());
            let diagnostics = analyze(&root, &cfg, &files, &sources, false).unwrap();
            assert_eq!(diagnostics.findings.len(), failures, "{rule}: {policy}");
            assert_eq!(diagnostics.advisories.len(), advisories, "{rule}: {policy}");
            assert!(diagnostics
                .findings
                .iter()
                .chain(&diagnostics.advisories)
                .all(|finding| finding.message.starts_with("Repair metadata:")));
            let deferred = analyze(&root, &cfg, &files, &sources, true).unwrap();
            let uncertain = deferred
                .findings
                .iter()
                .chain(&deferred.advisories)
                .filter(|finding| finding.target.as_deref() == Some("unanalyzable"))
                .count();
            assert_eq!(uncertain, if policy == "ignore" { 0 } else { 4 });
        }
        // Repeated policy queries borrow one store and reuse failure identities.
        assert_eq!(
            sources.physical_read_count(),
            files.len() + usize::from(rule == pnpm_overrides_ban::RULE_ID)
        );
    }
}

#[test]
fn unreadable_and_malformed_pnpm_inputs_are_not_clean() {
    for (rule, analyze) in [
        (
            pnpm_overrides_ban::RULE_ID,
            pnpm_overrides_ban::diagnostics_with_files_and_sources as Analyze,
        ),
        (
            pnpm_release_age_policy::RULE_ID,
            pnpm_release_age_policy::diagnostics_with_files_and_sources as Analyze,
        ),
    ] {
        for scenario in ["unreadable-workspace", "malformed-workspace"] {
            let root = fixture(rule, scenario);
            let files = [root.join("pnpm-workspace.yaml")];
            let sources = super::super::source_store_for_files(&files);
            let cfg = config(
                rule,
                "permanentPackages: [{name: acme, reason: first-party}]",
            );
            let diagnostics = analyze(&root, &cfg, &files, &sources, false).unwrap();
            assert_eq!(diagnostics.findings.len(), 1, "{rule} {scenario}");
            assert_eq!(
                diagnostics.findings[0].target.as_deref(),
                Some("unanalyzable")
            );
        }
    }
}

#[test]
fn selected_includes_report_failures_when_root_is_excluded() {
    let rule = pnpm_overrides_ban::RULE_ID;
    let root = fixture(rule, "unanalyzable-includes");
    let files = [
        root.join("pnpm/broken.yaml"),
        root.join("pnpm/invalid-utf8.yaml"),
    ];
    let sources = super::super::source_store_for_files(&files);
    let diagnostics = pnpm_overrides_ban::diagnostics_with_files_and_sources(
        &root,
        &config(rule, "{}"),
        &files,
        &sources,
        false,
    )
    .unwrap();
    assert_eq!(diagnostics.findings.len(), 2, "{:?}", diagnostics.findings);
    assert!(diagnostics
        .findings
        .iter()
        .all(|finding| finding.file.starts_with("pnpm/")
            && finding.target.as_deref() == Some("unanalyzable")));
    let selected = [
        files[0].clone(),
        files[1].clone(),
        root.join("pnpm-workspace.yaml"),
    ];
    let diagnostics = pnpm_overrides_ban::diagnostics_with_files_and_sources(
        &root,
        &config(rule, "{}"),
        &selected,
        &sources,
        false,
    )
    .unwrap();
    assert!(
        diagnostics
            .findings
            .iter()
            .any(|finding| finding.message.contains("invalid grant")),
        "{:?}",
        diagnostics.findings
    );
}

#[test]
fn release_age_optional_sources_report_present_failures_and_explicit_absence() {
    let rule = pnpm_release_age_policy::RULE_ID;
    for (scenario, path) in [
        ("unreadable-lockfile", "pnpm-lock.yaml"),
        ("unreadable-dependabot", ".github/dependabot.yml"),
        ("malformed-dependabot", ".github/dependabot.yml"),
    ] {
        let root = fixture(rule, scenario);
        let files = [root.join("pnpm-workspace.yaml"), root.join(path)];
        let sources = super::super::source_store_for_files(&files);
        let diagnostics = pnpm_release_age_policy::diagnostics_with_files_and_sources(
            &root,
            &config(
                rule,
                "permanentPackages: [{name: acme, reason: first-party}]",
            ),
            &files,
            &sources,
            false,
        )
        .unwrap();
        assert!(
            diagnostics
                .findings
                .iter()
                .any(|finding| finding.file == path
                    && finding.target.as_deref() == Some("unanalyzable")),
            "{:?}",
            diagnostics.findings
        );
    }
    let root = fixture(rule, "missing-input");
    let files = [root.join("package.json")];
    let sources = super::super::source_store_for_files(&files);
    let optional = config(
        rule,
        "permanentPackages: [{name: acme, reason: first-party}]",
    );
    assert!(pnpm_release_age_policy::diagnostics_with_files_and_sources(
        &root, &optional, &files, &sources, false
    )
    .unwrap()
    .findings
    .is_empty());
    let files = [root.join("package.json"), root.join("pnpm-workspace.yaml")];
    for option in ["workspaceYaml", "dependabotPath", "lockfilePath"] {
        // An explicit missing source is a request, rather than optional discovery.
        let options = format!("permanentPackages: [{{name: acme, reason: first-party}}]\n{option}: requested-missing.yaml");
        let diagnostics = pnpm_release_age_policy::diagnostics_with_files_and_sources(
            &root,
            &config(rule, &options),
            &files,
            &sources,
            false,
        )
        .unwrap();
        assert!(
            diagnostics
                .findings
                .iter()
                .any(|finding| finding.target.as_deref() == Some("unanalyzable")),
            "{option}"
        );
    }
}

#[test]
fn incomplete_manifest_names_do_not_prove_an_exemption_unused() {
    let rule = pnpm_release_age_policy::RULE_ID;
    let root = fixture(rule, "unanalyzable");
    let files = [
        root.join("pnpm-workspace.yaml"),
        root.join("packages/broken/package.json"),
    ];
    let sources = super::super::source_store_for_files(&files);
    let diagnostics = pnpm_release_age_policy::diagnostics_with_files_and_sources(
        &root,
        &config(
            rule,
            "permanentPackages: [{name: acme, reason: first-party}]",
        ),
        &files,
        &sources,
        false,
    )
    .unwrap();
    assert!(diagnostics
        .findings
        .iter()
        .any(|finding| finding.target.as_deref() == Some("unanalyzable")));
    assert!(!diagnostics
        .findings
        .iter()
        .any(|finding| finding.message.contains("registered but absent")));
    assert!(diagnostics
        .findings
        .iter()
        .any(|finding| finding.target.as_deref() == Some("unknown")));
}

#[test]
fn registry_explicit_lockfile_failures_use_the_same_policy() {
    let rule = package_json_registry_only::RULE_ID;
    for scenario in ["unreadable-lockfile", "missing-input"] {
        let root = fixture(rule, scenario);
        let sources = super::super::source_store_for_files(&[]);
        for (policy, failures, advice) in [("report", 1, 0), ("advise", 0, 1), ("ignore", 0, 0)] {
            let cfg = config(
                rule,
                &format!("lockfile: pnpm-lock.yaml\nunanalyzable: {policy}"),
            );
            let diagnostics = package_json_registry_only::diagnostics_with_files_and_sources(
                &root,
                &cfg,
                &[],
                &sources,
                false,
            )
            .unwrap();
            assert_eq!(diagnostics.findings.len(), failures);
            assert_eq!(diagnostics.advisories.len(), advice);
        }
        assert_eq!(sources.physical_read_count(), 1);
    }
}
