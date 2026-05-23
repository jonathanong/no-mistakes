use super::*;
use crate::codebase::check_facts::{CheckFactMap, CheckFileFacts};
use crate::config::v2::schema::{Project, ProjectType, RuleDef};
use std::collections::HashMap;

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/codebase-analysis/no-nextjs-api-routes")
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
fn reports_app_and_pages_api_routes() {
    let root = fixture();
    let findings = check(&root, &config()).unwrap();
    let files: Vec<_> = findings
        .iter()
        .map(|finding| finding.file.as_str())
        .collect();

    assert_eq!(
        files,
        vec![
            "web/app/api/users/route.ts",
            "web/pages/api/legacy.ts",
            "web/src/app/rss/route.ts",
            "web/src/pages/api/status.ts",
        ]
    );
}

#[test]
fn generic_runner_checks_nextjs_api_routes() {
    let root = fixture();
    let findings =
        crate::codebase::rules::run_check(&root, Some(&root.join(".no-mistakes.yml")), None)
            .unwrap();

    assert_eq!(findings.len(), 4);
}

#[test]
fn fact_runner_checks_nextjs_api_routes() {
    let root = fixture();
    let files = crate::codebase::ts_source::discover_files(&root, &[]);
    let facts = crate::codebase::check_facts::collect_check_facts(
        &root,
        files,
        crate::codebase::check_facts::CheckFactPlan {
            source: true,
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

    assert_eq!(findings.len(), 4);
}

#[test]
fn fact_runner_ignores_missing_source_outside_target_roots() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture());
    let outside = root.join("other/app/api/users/route.ts");
    let inside = root.join("web/app/api/users/route.ts");
    let facts = CheckFactMap {
        files: vec![outside.clone(), inside.clone()],
        ts: HashMap::from([
            (outside, CheckFileFacts::default()),
            (
                inside,
                CheckFileFacts {
                    source: Some("export function GET() {}".to_string()),
                    ..Default::default()
                },
            ),
        ]),
        ..Default::default()
    };
    let findings = check_with_facts(&root, &config(), &facts).unwrap();

    assert_eq!(findings.len(), 1);
}

#[test]
fn fact_runner_skips_missing_facts_inside_target_roots() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture());
    let inside = root.join("web/app/api/users/route.ts");
    let facts = CheckFactMap {
        files: vec![inside],
        ..Default::default()
    };
    let findings = check_with_facts(&root, &config(), &facts).unwrap();

    assert!(findings.is_empty());
}

#[test]
fn fact_runner_ignores_missing_source_for_non_route_target_files() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture());
    let inside = root.join("web/app/page.tsx");
    let facts = CheckFactMap {
        files: vec![inside.clone()],
        ts: HashMap::from([(inside, CheckFileFacts::default())]),
        ..Default::default()
    };
    let findings = check_with_facts(&root, &config(), &facts).unwrap();

    assert!(findings.is_empty());
}

#[test]
fn fact_runner_requires_source_for_target_files() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture());
    let inside = root.join("web/app/api/users/route.ts");
    let facts = CheckFactMap {
        files: vec![inside.clone()],
        ts: HashMap::from([(inside, CheckFileFacts::default())]),
        ..Default::default()
    };
    let err = check_with_facts(&root, &config(), &facts).unwrap_err();

    assert!(err.to_string().contains("requires source facts"), "{err:?}");
}

#[test]
fn fact_runner_reports_parse_errors_for_route_files() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture());
    let inside = root.join("web/app/api/users/route.ts");
    let facts = CheckFactMap {
        files: vec![inside.clone()],
        ts: HashMap::from([(
            inside,
            CheckFileFacts {
                parse_error: Some("failed to read fixture route".to_string()),
                ..Default::default()
            },
        )]),
        ..Default::default()
    };
    let err = check_with_facts(&root, &config(), &facts).unwrap_err();

    assert!(
        err.to_string().contains("failed to read fixture route"),
        "{err:?}"
    );
}

#[test]
fn direct_runner_reports_file_read_errors() {
    let root = fixture();
    let missing = root.join("web/app/api/missing/route.ts");
    let err = check_files(&root, &config(), &[missing]).unwrap_err();

    assert!(err.to_string().contains("failed to read"), "{err:?}");
}

#[test]
fn direct_runner_does_not_read_non_route_files() {
    let root = fixture();
    let missing = root.join("web/app/missing-page.tsx");
    let findings = check_files(&root, &config(), &[missing]).unwrap();

    assert!(findings.is_empty());
}

#[test]
fn route_matching_rejects_paths_outside_target_roots() {
    let root = fixture();
    let target_roots = vec![root.join("web")];
    let outside = root.join("other/app/api/users/route.ts");

    assert!(finding_for_file(&root, &target_roots, &outside, "").is_none());
    assert!(!is_nextjs_api_route(&outside, &target_roots));
}

#[test]
fn route_matching_rejects_non_route_paths_inside_target_roots() {
    let root = fixture();
    let target_roots = vec![root.join("web")];
    let inside = root.join("web/app/page.tsx");

    assert!(finding_for_file(&root, &target_roots, &inside, "").is_none());
    assert!(!is_nextjs_api_route(&inside, &target_roots));
}
