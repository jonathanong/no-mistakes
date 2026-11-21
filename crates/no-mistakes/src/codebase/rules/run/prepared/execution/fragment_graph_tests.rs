use super::super::PreparedRulesCheck;
use super::fragment_graph_facts::{consumer, exhausted, producer};
use super::run;
use crate::codebase::analysis_session::AnalysisSession;
use crate::codebase::check_facts::{CheckFactMap, CheckFileFacts};
use crate::config::v2::schema::{RuleDef, RuleScope};
use crate::config::v2::NoMistakesConfig;
use std::path::PathBuf;
use std::sync::Arc;

#[test]
fn incomplete_value_flow_keeps_fragment_escape_findings() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-sql-fragment-escape/fixtures/value-flow-gap"),
    );
    let columns = root.join("columns.ts");
    let extend = root.join("extend.ts");
    let wide = root.join("wide.ts");
    let source = std::fs::read_to_string(&extend).expect("saved extend.ts");
    let append = source.find("widgetColumns.append(extra)").expect("append");
    let start = u32::try_from(append).expect("offset");
    let end = start + u32::try_from("widgetColumns.append(extra)".len()).expect("end");
    let mut facts = CheckFactMap {
        graph_files_complete: true,
        graph_plan: crate::codebase::ts_source::facts::TsFactPlan {
            imports: true,
            function_calls: true,
            value_flow: true,
            ..crate::codebase::ts_source::facts::TsFactPlan::default()
        },
        ..CheckFactMap::default()
    };
    insert(&mut facts, columns, producer());
    insert(&mut facts, extend, consumer(&source, start, end));
    insert(&mut facts, wide, exhausted());
    let session = AnalysisSession::disabled();
    session.enable_partial_checks();
    let tsconfig = crate::codebase::ts_resolver::TsConfig {
        dir: root.clone(),
        paths_dir: root.clone(),
        ..crate::codebase::ts_resolver::TsConfig::default()
    };
    let catalog =
        crate::codebase::ts_resolver::TsConfigCatalog::forced(&root, tsconfig.clone(), None);
    let config = fragment_config();
    let findings = run(
        PreparedRulesCheck {
            session: Arc::clone(&session),
            root: &root,
            config_path: None,
            tsconfig_path: None,
            shared: &facts,
            prepared_playwright: None,
            config: &config,
            prepared_graph: None,
            prepared_tsconfig: &tsconfig,
            prepared_tsconfig_catalog: &catalog,
            prepared_vitest_projects: None,
            prepared_playwright_projects: None,
            inferred_roots: None,
            sources: None,
        },
        None,
        None,
        false,
    )
    .expect("fragment escape runs when one file exhausts value flow")
    .findings;
    assert!(
        findings.iter().any(|finding| {
            finding.rule == "postgres-sql-fragment-escape"
                && finding.file == "extend.ts"
                && finding.target.as_deref() == Some("widgetColumns")
        }),
        "{findings:?}"
    );
    let gaps = session.check_failures();
    assert!(
        gaps.iter().any(|gap| {
            gap.consumer == "postgres-sql-fragment-escape"
                && gap.message.contains("wide.ts")
                && gap.message.contains("incomplete (analysis budget)")
        }),
        "{gaps:?}"
    );
}

fn fragment_config() -> NoMistakesConfig {
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: "postgres-sql-fragment-escape".to_string(),
        scope: Some(RuleScope::Repository),
        options: serde_yaml::from_str(
            "trustedSqlTags:\n  - {module: \"@example/sql\", name: default}",
        )
        .expect("options"),
        ..RuleDef::default()
    });
    config
}

fn insert(
    facts: &mut CheckFactMap,
    file: PathBuf,
    ts: crate::codebase::ts_source::facts::TsFileFacts,
) {
    facts.graph_files.push(file.clone());
    facts.ts.insert(
        file,
        Arc::new(CheckFileFacts {
            ts: Arc::new(ts),
            ..CheckFileFacts::default()
        }),
    );
}
