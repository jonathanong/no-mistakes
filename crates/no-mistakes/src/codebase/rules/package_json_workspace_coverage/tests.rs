use super::patterns::{build_workspace_globset, normalize_relative_path, unmatched_patterns};
use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::Path;

fn package_name(path: &Path) -> Option<String> {
    let sources = crate::codebase::rules::source_store_for_files(&[path.to_path_buf()]);
    package_name_with_sources(path, &sources)
}

fn fixture_root(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/package-json-workspace-coverage")
            .join(name),
    )
}

fn config(yaml: &str) -> NoMistakesConfig {
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: RULE_ID.to_string(),
        scope: Some(RuleScope::Repository),
        options: serde_yaml::from_str(yaml).unwrap(),
        ..Default::default()
    });
    config
}

#[test]
fn reports_package_json_under_configured_roots_missing_from_workspaces() {
    let root = fixture_root("missing");
    let files = vec![
        root.join("package.json"),
        root.join("packages/admin/package.json"),
        root.join("packages/api/package.json"),
        root.join("packages/web/package.json"),
    ];
    let findings = check_with_files(
        &root,
        &config("packageRoots: [./packages]\nrequireNamedPackage: true\n"),
        &files,
    )
    .unwrap();

    assert_eq!(findings.len(), 2);
    assert_eq!(findings[0].file, "packages/admin/package.json");
    assert_eq!(findings[1].file, "packages/web/package.json");
    assert!(findings[0].message.contains("not covered"));
}

#[test]
fn allowlist_suppresses_known_non_workspace_packages() {
    let root = fixture_root("missing");
    let files = vec![
        root.join("package.json"),
        root.join("packages/api/package.json"),
        root.join("packages/web/package.json"),
    ];
    let findings = check_with_files(
        &root,
        &config(
            "packageRoots: [packages]\nallowlist: [packages/web/package.json]\nrequireNamedPackage: true\n",
        ),
        &files,
    )
    .unwrap();

    assert!(findings.is_empty(), "unexpected findings: {findings:?}");
}

#[test]
fn allowlist_normalizes_relative_package_json_paths() {
    let root = fixture_root("missing");
    let files = vec![
        root.join("package.json"),
        root.join("packages/api/package.json"),
        root.join("packages/web/package.json"),
    ];
    let findings = check_with_files(
        &root,
        &config(
            "packageRoots: [packages]\nallowlist: [./packages/web/package.json]\nrequireNamedPackage: true\n",
        ),
        &files,
    )
    .unwrap();

    assert!(findings.is_empty(), "unexpected findings: {findings:?}");
}

#[test]
fn require_named_package_reports_unnamed_manifests() {
    let root = fixture_root("missing");
    let files = vec![
        root.join("package.json"),
        root.join("packages/api/package.json"),
        root.join("packages/unnamed/package.json"),
        root.join("packages/unnamed/index.ts"),
    ];
    let findings = check_with_files(
        &root,
        &config("packageRoots: [packages]\nrequireNamedPackage: true\n"),
        &files,
    )
    .unwrap();

    assert!(
        findings.iter().any(|finding| {
            finding.file == "packages/unnamed/package.json"
                && finding.message.contains("must declare a name")
        }),
        "{findings:?}"
    );
    assert_eq!(
        package_name(&root.join("packages/unnamed/package.json")),
        None
    );
}

#[test]
fn require_named_package_does_not_report_malformed_json_as_unnamed() {
    let root = fixture_root("invalid-package-json");
    let files = vec![
        root.join("package.json"),
        root.join("packages/bad/package.json"),
    ];
    let findings = check_with_files(
        &root,
        &config("packageRoots: [packages]\nrequireNamedPackage: true\n"),
        &files,
    )
    .unwrap();

    assert!(
        findings
            .iter()
            .all(|finding| !finding.message.contains("must declare a name")),
        "{findings:?}"
    );
    assert_eq!(package_name(&root.join("packages/bad/package.json")), None);
}

#[test]
fn require_named_package_does_not_treat_star_as_crossing_directories() {
    let root = fixture_root("unnamed-covered");
    let files = vec![
        root.join("package.json"),
        root.join("packages/unnamed/package.json"),
        root.join("packages/group/app/package.json"),
    ];
    let findings = check_with_files(
        &root,
        &config("packageRoots: [packages]\nrequireNamedPackage: true\n"),
        &files,
    )
    .unwrap();

    assert!(
        findings.iter().any(|finding| {
            finding.file == "packages/group/app/package.json"
                && finding.message.contains("not covered")
        }),
        "{findings:?}"
    );
    assert!(
        findings.iter().any(|finding| {
            finding.file == "packages/unnamed/package.json"
                && finding.message.contains("must declare a name")
        }),
        "{findings:?}"
    );
    assert!(
        findings.iter().all(|finding| {
            finding.file != "packages/unnamed/package.json"
                || !finding.message.contains("not covered")
        }),
        "{findings:?}"
    );
}

#[test]
fn nested_package_is_uncovered_when_names_are_not_required() {
    let root = fixture_root("unnamed-covered");
    let files = vec![
        root.join("package.json"),
        root.join("packages/unnamed/package.json"),
        root.join("packages/group/app/package.json"),
    ];
    let findings = check_with_files(&root, &config("packageRoots: [packages]\n"), &files).unwrap();

    assert!(
        findings.iter().any(|finding| {
            finding.file == "packages/group/app/package.json"
                && finding.message.contains("not covered")
        }),
        "{findings:?}"
    );
    assert!(
        findings
            .iter()
            .all(|finding| finding.file != "packages/unnamed/package.json"),
        "{findings:?}"
    );
}

#[test]
fn workspace_globset_builder_splits_includes_and_excludes() {
    let patterns = vec![
        "./packages/*".to_string(),
        "!./packages/internal".to_string(),
    ];
    let include = build_workspace_globset(&patterns, false).unwrap();
    let exclude = build_workspace_globset(&patterns, true).unwrap();

    assert!(include.is_match("packages/web"));
    assert!(include.is_match("packages/internal"));
    assert!(!include.is_match("packages/group/app"));
    assert!(exclude.is_match("packages/internal"));
    assert!(!exclude.is_match("packages/web"));
    assert!(build_workspace_globset(&["[".to_string()], false).is_err());
    assert_eq!(
        normalize_relative_path("./packages/../packages/web"),
        "packages/web"
    );
    assert_eq!(
        normalize_relative_path("*/../packages/*"),
        "*/../packages/*"
    );
}

fn files_named(root: &Path, names: &[&str]) -> Vec<PathBuf> {
    names.iter().map(|name| root.join(name)).collect()
}

#[test]
fn positive_patterns_match_alone_and_invalid_globs_fail() {
    let missing = unmatched_patterns(
        &[
            "packages/*".to_string(),
            "apps/*".to_string(),
            "!packages/hidden".to_string(),
        ],
        &["packages/hidden".to_string()],
    )
    .unwrap();
    assert_eq!(missing, vec!["apps/*".to_string()]);
    assert!(unmatched_patterns(&["[".to_string()], &[]).is_err());
}

#[test]
fn unmatched_workspace_entries_ignore_negated_patterns() {
    let root = fixture_root("unmatched-pattern");
    let findings = check_with_files(
        &root,
        &config("packageRoots: [packages, apps]\n"),
        &files_named(
            &root,
            &[
                "pnpm-workspace.yaml",
                "packages/hidden/package.json",
                "packages/group/app/package.json",
            ],
        ),
    )
    .unwrap();
    let body = findings
        .iter()
        .map(|finding| finding.message.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        body.contains("workspace pattern \"apps/*\" does not correspond to a package directory"),
        "{body}"
    );
    assert!(!body.contains("workspace pattern \"packages/*\""), "{body}");
    assert!(!body.contains("workspace pattern \"!"), "{body}");
    assert!(body.contains("packages/hidden/package.json"), "{body}");
    assert!(body.contains("packages/group/app/package.json"), "{body}");
    assert_eq!(findings[0].file, "packages/group/app/package.json");
}

#[test]
fn unmatched_package_json_workspaces_attach_to_the_root_manifest() {
    let root = fixture_root("unmatched-package-json");
    let findings = check_with_files(
        &root,
        &config("packageRoots: [packages]\n"),
        &files_named(&root, &["package.json", "packages/hidden/package.json"]),
    )
    .unwrap();
    assert!(
        findings.iter().any(|finding| {
            finding.file == "package.json"
                && finding.message.contains("workspace pattern \"apps/*\"")
        }),
        "{findings:?}"
    );
    assert!(
        findings
            .iter()
            .all(|finding| !finding.message.contains("workspace pattern \"packages/*\"")),
        "{findings:?}"
    );
}

#[test]
fn included_package_globs_are_relative_to_the_included_file() {
    let root = fixture_root("included-packages");
    let findings = check_with_files(
        &root,
        &config("packageRoots: [packages, services]\n"),
        &files_named(
            &root,
            &[
                "pnpm-workspace.yaml",
                "pnpm/packages.yaml",
                "packages/api/package.json",
                "packages/group/app/package.json",
                "services/package.json",
            ],
        ),
    )
    .unwrap();
    let files: Vec<_> = findings
        .iter()
        .map(|finding| finding.file.as_str())
        .collect();
    assert_eq!(
        files,
        vec!["packages/group/app/package.json", "services/package.json"]
    );
}

#[test]
fn duplicate_package_settings_keep_the_root_coverage_finding() {
    let root = fixture_root("duplicate-packages");
    let findings = check_with_files(
        &root,
        &config("packageRoots: [packages, services]\n"),
        &files_named(
            &root,
            &[
                "pnpm-workspace.yaml",
                "pnpm/packages.yaml",
                "packages/api/package.json",
                "services/package.json",
            ],
        ),
    )
    .unwrap();
    let files: Vec<_> = findings
        .iter()
        .map(|finding| finding.file.as_str())
        .collect();
    assert_eq!(
        files,
        vec!["pnpm-workspace.yaml", "services/package.json"],
        "{findings:?}"
    );
    assert!(findings.iter().any(|finding| {
        finding.file == "pnpm-workspace.yaml"
            && finding
                .message
                .contains("setting \"packages\" is claimed by more than one pnpm source")
    }));
    assert!(findings.iter().any(|finding| {
        finding.file == "services/package.json"
            && finding.message.contains("package directory is not covered")
    }));
}

#[test]
fn narrowed_scope_does_not_guess_coverage_for_duplicate_packages() {
    let root = fixture_root("duplicate-packages");
    let mut narrowed = config("packageRoots: [packages, services]\n");
    narrowed.rules[0].include = vec!["packages/**".to_string()];
    let findings = check_with_files(
        &root,
        &narrowed,
        &files_named(
            &root,
            &[
                "pnpm-workspace.yaml",
                "pnpm/packages.yaml",
                "packages/api/package.json",
                "services/package.json",
            ],
        ),
    )
    .unwrap();
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn delegated_omitted_packages_do_not_fall_back_to_star() {
    let root = fixture_root("delegated-omitted");
    let findings = check_with_files(
        &root,
        &config("packageRoots: [packages, services]\n"),
        &files_named(
            &root,
            &[
                "pnpm-workspace.yaml",
                "pnpm/packages.yaml",
                "packages/api/package.json",
                "services/package.json",
            ],
        ),
    )
    .unwrap();
    let files: Vec<_> = findings
        .iter()
        .map(|finding| finding.file.as_str())
        .collect();
    assert_eq!(
        files,
        vec!["packages/api/package.json", "services/package.json"]
    );
}

#[test]
fn unreadable_package_include_is_a_finding_and_does_not_hide_packages() {
    let root = fixture_root("unreadable-packages");
    let findings = check_with_files(
        &root,
        &config("packageRoots: [packages, services]\n"),
        &files_named(
            &root,
            &[
                "pnpm-workspace.yaml",
                "packages/api/package.json",
                "services/package.json",
            ],
        ),
    )
    .unwrap();
    let body = format!("{findings:?}");
    assert!(body.contains("could not be read"), "{body}");
    assert!(body.contains("packages/api/package.json"), "{body}");
    assert!(body.contains("services/package.json"), "{body}");
}

#[test]
fn included_scope_does_not_mark_an_existing_outside_package_unmatched() {
    let root = fixture_root("scoped-app");
    let mut narrowed = config("packageRoots: [packages]\n");
    narrowed.rules[0].include = vec!["packages/**".to_string(), "pnpm-workspace.yaml".to_string()];
    let present = files_named(
        &root,
        &[
            "pnpm-workspace.yaml",
            "packages/api/package.json",
            "apps/web/package.json",
        ],
    );
    let findings = check_with_files(&root, &narrowed, &present).unwrap();
    assert!(
        findings
            .iter()
            .all(|finding| !finding.message.contains("apps/*")),
        "{findings:?}"
    );
    let without_app = files_named(&root, &["pnpm-workspace.yaml", "packages/api/package.json"]);
    let missing = check_with_files(&root, &narrowed, &without_app).unwrap();
    assert!(
        missing
            .iter()
            .any(|finding| finding.message.contains("apps/*")),
        "{missing:?}"
    );
}

#[test]
fn excluded_workspace_file_does_not_report_unmatched_patterns() {
    let root = fixture_root("scoped-app");
    let mut narrowed = config("packageRoots: [packages]\n");
    narrowed.rules[0].include = vec!["packages/**".to_string()];
    let files = files_named(&root, &["pnpm-workspace.yaml", "packages/api/package.json"]);
    let findings = check_with_files(&root, &narrowed, &files).unwrap();
    assert!(
        findings
            .iter()
            .all(|finding| !finding.message.contains("apps/*") && finding.file != "package.json"),
        "{findings:?}"
    );
}

#[test]
fn invalid_workspace_globs_fail_the_check() {
    let root = fixture_root("invalid-glob");
    let error = check_with_files(
        &root,
        &config("packageRoots: [packages]\n"),
        &files_named(&root, &["pnpm-workspace.yaml", "packages/api/package.json"]),
    )
    .unwrap_err();
    assert!(!error.to_string().is_empty());
}

#[test]
fn invalid_packages_field_fails_the_check() {
    let root = fixture_root("invalid-packages");
    let error = check_with_files(
        &root,
        &config("packageRoots: [packages]\n"),
        &files_named(&root, &["pnpm-workspace.yaml", "packages/api/package.json"]),
    )
    .unwrap_err();
    assert!(error
        .to_string()
        .contains("packages field must be a list of strings"));
}
