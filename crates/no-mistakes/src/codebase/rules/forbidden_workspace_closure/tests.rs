use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};

fn fixture_root(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/forbidden-workspace-closure")
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

fn package_files(root: &Path, files: &[&str]) -> Vec<PathBuf> {
    files.iter().map(|file| root.join(file)).collect()
}

#[test]
fn reports_direct_external_dependency() {
    let root = fixture_root("direct-external");
    let files = package_files(
        &root,
        &[
            "package.json",
            "packages/app/package.json",
            "packages/domain/package.json",
        ],
    );

    let findings = check_with_files(
        &root,
        &config("packages: [\"@acme/app\"]\nforbidden: [\"@acme/secret\"]\n"),
        &files,
    )
    .unwrap();

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, "packages/app/package.json");
    assert_eq!(findings[0].target.as_deref(), Some("@acme/secret"));
    assert_eq!(
        findings[0].import.as_deref(),
        Some("@acme/app -> @acme/secret")
    );
}

#[test]
fn reports_transitive_external_dependency() {
    let root = fixture_root("transitive-workspace");
    let files = package_files(
        &root,
        &[
            "package.json",
            "packages/app/package.json",
            "packages/domain/package.json",
        ],
    );

    let findings = check_with_files(
        &root,
        &config("packages: [\"@acme/app\"]\nforbidden: [\"@acme/secret\"]\n"),
        &files,
    )
    .unwrap();

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, "packages/domain/package.json");
    assert_eq!(findings[0].target.as_deref(), Some("@acme/secret"));
    assert_eq!(
        findings[0].import.as_deref(),
        Some("@acme/app -> @acme/domain -> @acme/secret")
    );
}

#[test]
fn passes_when_forbidden_package_is_outside_closure() {
    let root = fixture_root("pass");
    let files = package_files(
        &root,
        &[
            "package.json",
            "packages/app/package.json",
            "packages/api/package.json",
            "packages/domain/package.json",
        ],
    );

    let findings = check_with_files(
        &root,
        &config("packages: [\"@acme/app\"]\nforbidden: [\"@acme/secret\"]\n"),
        &files,
    )
    .unwrap();

    assert!(findings.is_empty(), "unexpected findings: {findings:?}");
}

#[test]
fn dependency_types_control_dev_dependency_closure() {
    let root = fixture_root("dependency-types");
    let files = package_files(
        &root,
        &[
            "package.json",
            "packages/app/package.json",
            "packages/domain/package.json",
        ],
    );

    let default_findings = check_with_files(
        &root,
        &config("packages: [\"@acme/app\"]\nforbidden: [\"@acme/secret\"]\n"),
        &files,
    )
    .unwrap();
    assert!(default_findings.is_empty(), "{default_findings:?}");

    let dev_findings = check_with_files(
        &root,
        &config(
            "packages: [\"@acme/app\"]\nforbidden: [\"@acme/secret\"]\ndependencyTypes: [dependencies, devDependencies]\n",
        ),
        &files,
    )
    .unwrap();
    assert_eq!(dev_findings.len(), 1);
    assert_eq!(
        dev_findings[0].import.as_deref(),
        Some("@acme/app -> @acme/domain -> @acme/secret")
    );
}

#[test]
fn glob_pattern_matches_forbidden_package_name() {
    let root = fixture_root("glob");
    let files = package_files(
        &root,
        &[
            "package.json",
            "packages/app/package.json",
            "packages/infra-secret/package.json",
        ],
    );

    let findings = check_with_files(
        &root,
        &config("packages: [\"@acme/app\"]\nforbidden: [\"@acme/*secret\"]\n"),
        &files,
    )
    .unwrap();

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].target.as_deref(), Some("@acme/infra-secret"));
    assert_eq!(
        findings[0].import.as_deref(),
        Some("@acme/app -> @acme/infra-secret")
    );
}

#[test]
fn cycle_does_not_prevent_finding() {
    let root = fixture_root("cycle");
    let files = package_files(
        &root,
        &[
            "package.json",
            "packages/app/package.json",
            "packages/domain/package.json",
        ],
    );

    let findings = check_with_files(
        &root,
        &config("packages: [\"@acme/app\"]\nforbidden: [\"@acme/secret\"]\n"),
        &files,
    )
    .unwrap();

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].target.as_deref(), Some("@acme/secret"));
    assert_eq!(
        findings[0].import.as_deref(),
        Some("@acme/app -> @acme/domain -> @acme/secret")
    );
}

#[test]
fn pnpm_lockfile_alias_resolution_name_is_forbidden() {
    let root = fixture_root("lockfile-alias");
    let files = package_files(&root, &["package.json", "packages/app/package.json"]);

    let findings = check_with_files(
        &root,
        &config(
            "packages: [\"@acme/app\"]\nforbidden: [\"@acme/secret\"]\nlockfile: pnpm-lock.yaml\n",
        ),
        &files,
    )
    .unwrap();

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, "packages/app/package.json");
    assert_eq!(findings[0].target.as_deref(), Some("@acme/secret"));
    assert_eq!(
        findings[0].import.as_deref(),
        Some("@acme/app -> @acme/secret")
    );
}

#[test]
fn missing_options_emit_config_finding() {
    let root = fixture_root("pass");
    let files = package_files(
        &root,
        &[
            "package.json",
            "packages/app/package.json",
            "packages/domain/package.json",
        ],
    );
    let findings = check_with_files(&root, &config("{}"), &files).unwrap();

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, ".no-mistakes.yml");
    assert!(findings[0].message.contains("packages"));
    assert!(findings[0].message.contains("forbidden"));
}
