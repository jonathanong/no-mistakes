use super::*;
use crate::config::v2::schema::{Project, ProjectType, RuleDef};

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/codebase-analysis/no-nextjs-caching")
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
            "web/app/bad.ts",
            "web/app/bad.ts",
            "web/app/directive.ts",
            "web/app/fetch-options.ts",
            "web/app/fetch-options.ts",
            "web/app/fetch-options.ts",
            "web/app/fetch-options.ts",
            "web/app/segment-config.ts",
            "web/app/segment-config.ts",
            "web/app/segment-config.ts",
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
}

#[test]
fn generic_runner_checks_nextjs_caching() {
    let root = fixture();
    let findings =
        crate::codebase::rules::run_check(&root, Some(&root.join(".no-mistakes.yml")), None)
            .unwrap();

    assert_eq!(findings.len(), 16);
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

    assert_eq!(findings.len(), 16);
}

#[test]
fn extract_allows_uncached_fetch_and_dynamic_segment_config() {
    let source = "export const dynamic = 'force-dynamic'\n\
export const revalidate = 0\n\
export async function load() {\n\
  await fetch('/api/a', { cache: 'no-store' })\n\
  await fetch('/api/b', { next: { revalidate: 0 } })\n\
}\n";
    let findings = extract(Path::new("app/good.ts"), source).unwrap();

    assert!(findings.is_empty());
}
