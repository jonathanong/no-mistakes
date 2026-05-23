use super::*;
use crate::codebase::check_facts::{CheckFactMap, CheckFileFacts};
use crate::config::v2::schema::{Project, ProjectType, RuleDef};
use std::collections::HashMap;

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
fn fact_runner_ignores_missing_facts_outside_target_roots() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture());
    let outside = root.join("other/app/bad.ts");
    let facts = CheckFactMap {
        files: vec![outside.clone()],
        ts: HashMap::from([(outside, CheckFileFacts::default())]),
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
        ts: HashMap::from([(inside.clone(), CheckFileFacts::default())]),
        ..Default::default()
    };
    let err = check_with_facts(&root, &config(), &missing_source).unwrap_err();
    assert!(err.to_string().contains("requires source facts"), "{err:?}");

    let missing_cache = CheckFactMap {
        files: vec![inside.clone()],
        ts: HashMap::from([(
            inside,
            CheckFileFacts {
                source: Some("export const value = 1".to_string()),
                ..Default::default()
            },
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

#[test]
fn extract_reports_top_level_cache_directive() {
    let source = "'use cache'\nexport const value = 1\n";
    let findings = extract(Path::new("app/directive.ts"), source).unwrap();

    assert_eq!(findings.len(), 1);
    assert!(findings
        .iter()
        .any(|finding| finding.message.contains("cache directives")));
}

#[test]
fn extract_ignores_unrelated_unstable_cache_method() {
    let source = "const local = { unstable_cache() { return 1 } }\n\
export const value = local.unstable_cache()\n";
    let findings = extract(Path::new("app/local.ts"), source).unwrap();

    assert!(findings.is_empty());
}

#[test]
fn extract_reports_next_cache_namespace_unstable_cache_call() {
    let source = "import * as cache from 'next/cache'\n\
export const value = cache.unstable_cache(async () => 1)\n";
    let findings = extract(Path::new("app/cache.ts"), source).unwrap();

    assert!(findings
        .iter()
        .any(|finding| finding.message.contains("namespace imports")));
    assert!(findings
        .iter()
        .any(|finding| finding.message.contains("unstable_cache")));
}

#[test]
fn extract_reports_next_cache_default_import() {
    let source = "import cache from 'next/cache'\n";
    let findings = extract(Path::new("app/cache.ts"), source).unwrap();

    assert_eq!(findings.len(), 1);
    assert!(findings
        .iter()
        .any(|finding| finding.message.contains("default imports")));
}

#[test]
fn extract_reports_next_cache_side_effect_import() {
    let source = "import 'next/cache'\n";
    let findings = extract(Path::new("app/cache.ts"), source).unwrap();

    assert_eq!(findings.len(), 1);
    assert!(findings
        .iter()
        .any(|finding| finding.message.contains("side-effect imports")));
}

#[test]
fn extract_ignores_dynamic_or_uncached_option_shapes() {
    let source = "const key = 'cache'\n\
const opts = {}\n\
export const cacheComponents = 'no'\n\
export const fetchCache = 'auto'\n\
export const dynamic = 'auto'\n\
export async function load() {\n\
  await fetch('/api/a', { ...opts, [key]: 'force-cache', next: opts })\n\
  await fetch('/api/b', { next: { ...opts, [key]: 1, revalidate: true } })\n\
}\n\
export default { ...opts, [key]: true, cacheComponents: 'yes' }\n";
    let findings = extract(Path::new("app/good.ts"), source).unwrap();

    assert!(findings.is_empty());
}

#[test]
fn extract_ignores_non_cache_export_and_import_shapes() {
    let source = "import { notCache } from 'next/cache'\n\
const cfg = {}\n\
const { cacheLife } = cfg\n\
let nextConfig\n\
export const { dynamic } = cfg\n\
export let revalidate\n\
export const fetchCache = 1\n\
export const dynamicValue = 'force-static'\n\
export default function config() {}\n";
    let findings = extract(Path::new("next.config.ts"), source).unwrap();

    assert!(findings.is_empty());
}

#[test]
fn extract_reports_wrapped_next_config_object() {
    let source = "export default withSentryConfig({\n\
  cacheComponents: true,\n\
  cacheHandlers: {},\n\
})\n";
    let findings = extract(Path::new("next.config.ts"), source).unwrap();

    assert_eq!(findings.len(), 2);
    assert!(findings
        .iter()
        .any(|finding| finding.message.contains("cacheComponents")));
    assert!(findings
        .iter()
        .any(|finding| finding.message.contains("cacheHandlers")));
}

#[test]
fn extract_reports_identifier_next_config_object() {
    let source = "const nextConfig = {\n\
  cacheComponents: true,\n\
  cacheLife: {},\n\
}\n\
export default nextConfig\n";
    let findings = extract(Path::new("next.config.ts"), source).unwrap();

    assert_eq!(findings.len(), 2);
    assert!(findings
        .iter()
        .any(|finding| finding.message.contains("cacheComponents")));
    assert!(findings
        .iter()
        .any(|finding| finding.message.contains("cacheLife")));
}
