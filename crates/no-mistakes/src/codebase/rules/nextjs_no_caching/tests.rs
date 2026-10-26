macro_rules! record_aggregate_scope_check {
    () => {
        tests::record_scope_check()
    };
}

thread_local! {
    static AGGREGATE_SCOPE_CHECKS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

pub(super) fn record_scope_check() {
    AGGREGATE_SCOPE_CHECKS.with(|count| count.set(count.get() + 1));
}

fn take_scope_checks() -> usize {
    AGGREGATE_SCOPE_CHECKS.with(|count| count.replace(0))
}

mod empty_facts;

use super::*;

fn check_with_facts(
    root: &Path,
    config: &NoMistakesConfig,
    facts: &CheckFactMap,
) -> anyhow::Result<Vec<RuleFinding>> {
    diagnostics_with_facts_for_aggregate(root, config, facts, None, false)
        .map(|diagnostics| diagnostics.findings)
}
use crate::codebase::check_facts::{CheckFactMap, CheckFileFacts};
use crate::codebase::ts_source::FileIdMap;
use crate::config::v2::schema::{Project, ProjectType, RuleDef};

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/no-nextjs-caching/fixture")
}

fn config() -> NoMistakesConfig {
    let mut config = NoMistakesConfig::default();
    config.projects.insert(
        "web".to_string(),
        Project {
            type_: Some(ProjectType::Nextjs),
            root: Some("web".to_string()),
            ..Default::default()
        },
    );
    config.rules.push(RuleDef {
        rule: RULE_ID.to_string(),
        projects: vec!["web".to_string()],
        ..Default::default()
    });
    config
}

#[test]
fn const_scope_ignores_records_when_no_scope_is_open() {
    let mut scopes = Vec::new();
    super::const_values::record_unknown(&mut scopes, "mode");
    super::const_values::pop_scope(&mut scopes);
    assert!(scopes.is_empty());
}

#[test]
fn reports_nextjs_cache_surfaces() {
    let root = fixture();
    let findings = check(&root, &config()).unwrap();
    let files: Vec<_> = findings
        .iter()
        .map(|finding| finding.file.as_str())
        .collect();

    assert_eq!(
        files,
        vec![
            "web/app/bad.ts",
            "web/app/bad.ts",
            "web/app/bad.ts",
            "web/app/directive.ts",
            "web/app/fetch-options.ts",
            "web/app/fetch-options.ts",
            "web/app/fetch-options.ts",
            "web/app/fetch-options.ts",
            "web/app/static-keys/page.tsx",
            "web/app/static-keys/page.tsx",
            "web/app/static-keys/page.tsx",
            "web/app/static-keys/page.tsx",
            "web/app/static-keys/page.tsx",
            "web/app/static-keys/page.tsx",
            "web/app/static-keys/page.tsx",
            "web/app/static-keys/page.tsx",
            "web/app/static-keys/page.tsx",
            "web/app/static-keys/page.tsx",
            "web/next.config.ts",
            "web/next.config.ts",
            "web/next.config.ts",
        ]
    );
    assert!(findings
        .iter()
        .any(|finding| finding.message.contains("unstable_cache")));
    assert!(findings
        .iter()
        .any(|finding| finding.message.contains("force-cache")));
}

#[test]
fn allows_no_cache_forms_and_disable_comments() {
    let root = fixture();
    let findings = check(&root, &config()).unwrap();

    assert!(!findings
        .iter()
        .any(|finding| finding.file == "web/app/good.ts"));
    assert!(!findings
        .iter()
        .any(|finding| finding.file == "web/app/disabled.ts"));
    assert!(!findings
        .iter()
        .any(|finding| finding.file == "web/app/next-line-disabled.ts"));
    assert!(
        !findings
            .iter()
            .any(|finding| finding.file == "web/app/static-key-limits/page.tsx"),
        "{findings:?}"
    );
}

#[test]
fn generic_runner_checks_nextjs_caching() {
    let root = fixture();
    let findings =
        crate::codebase::rules::run_check(&root, Some(&root.join(".no-mistakes.yml")), None)
            .unwrap();

    assert_eq!(
        findings.len(),
        21,
        "{:?}",
        findings
            .iter()
            .map(|finding| format!("{}:{} {}", finding.file, finding.line, finding.message))
            .collect::<Vec<_>>()
    );
}

#[test]
fn fact_runner_checks_nextjs_caching() {
    let root = fixture();
    let files = crate::codebase::ts_source::discover_files(&root, &[]);
    let facts = crate::codebase::check_facts::collect_check_facts(
        &root,
        files,
        crate::codebase::check_facts::CheckFactPlan {
            source: true,
            nextjs_caching: true,
            ..Default::default()
        },
    );
    let findings = crate::codebase::rules::run_check_with_facts(
        &root,
        Some(&root.join(".no-mistakes.yml")),
        None,
        &facts,
    )
    .unwrap();

    assert_eq!(findings.len(), 21);
}

#[test]
fn fact_runner_ignores_missing_facts_outside_target_roots() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture());
    let outside = root.join("other/app/bad.ts");
    let facts = CheckFactMap {
        files: vec![outside.clone()],
        ts: FileIdMap::from([(outside, std::sync::Arc::new(CheckFileFacts::default()))]),
        ..Default::default()
    };
    let findings = check_with_facts(&root, &config(), &facts).unwrap();

    assert!(findings.is_empty());
}

#[test]
fn fact_runner_requires_source_and_cache_facts_for_target_files() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture());
    let inside = root.join("web/app/bad.ts");
    let missing_source = CheckFactMap {
        files: vec![inside.clone()],
        ts: FileIdMap::from([(
            inside.clone(),
            std::sync::Arc::new(CheckFileFacts::default()),
        )]),
        ..Default::default()
    };
    let err = check_with_facts(&root, &config(), &missing_source).unwrap_err();
    assert!(err.to_string().contains("requires source facts"), "{err:?}");

    let missing_cache = CheckFactMap {
        files: vec![inside.clone()],
        ts: FileIdMap::from([(
            inside,
            CheckFileFacts {
                source: Some("export const value = 1".into()),
                ..Default::default()
            }
            .into(),
        )]),
        ..Default::default()
    };
    let err = check_with_facts(&root, &config(), &missing_cache).unwrap_err();
    assert!(
        err.to_string().contains("requires Next.js caching facts"),
        "{err:?}"
    );
}

#[test]
fn direct_runner_ignores_unreadable_and_unparseable_files() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture());
    let findings = check_files(
        &root,
        &config(),
        &[
            root.join("web/app/missing.ts"),
            root.join("web/app/invalid.ts"),
        ],
    )
    .unwrap();

    assert!(findings.is_empty());
}

mod config_shapes;
mod coverage;
mod extraction;
mod nested;
mod shadowing;

mod effort;
