use super::*;
use crate::config::v2::schema::{Project, ProjectType, RuleDef};

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
