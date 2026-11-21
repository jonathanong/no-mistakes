use super::super::PreparedRulesCheck;
use super::run;
use crate::codebase::analysis_session::AnalysisSession;
use crate::codebase::check_facts::{CheckFactMap, CheckFileFacts};
use crate::codebase::ts_source::facts::{collect_ts_facts, TsFactPlan};
use std::path::PathBuf;
use std::sync::Arc;

#[test]
fn incomplete_value_flow_keeps_fragment_escape_without_sibling_findings() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-sql-fragment-escape/fixtures/sibling-graph"),
    );
    let files: Vec<PathBuf> = [
        "columns.ts",
        "extend.ts",
        "items.ts",
        "orphan-error.ts",
        "place-order.ts",
        "sql.ts",
        "widget-error.ts",
        "wide.ts",
    ]
    .into_iter()
    .map(|name| root.join(name))
    .collect();
    let collected = collect_ts_facts(&files, fact_plan());
    let wide = root.join("wide.ts");
    assert!(
        collected[&wide].value_flow.incomplete_reason.is_some(),
        "wide.ts must exhaust value flow"
    );
    assert!(collected[&root.join("widget-error.ts")]
        .value_flow
        .incomplete_reason
        .is_none());
    let mut facts = CheckFactMap {
        graph_files_complete: true,
        graph_plan: fact_plan(),
        ..CheckFactMap::default()
    };
    for (file, ts) in collected {
        facts.graph_files.push(file.clone());
        facts.ts.insert(
            file,
            Arc::new(CheckFileFacts {
                ts: Arc::new(ts),
                ..CheckFileFacts::default()
            }),
        );
    }
    let session = AnalysisSession::disabled();
    session.enable_partial_checks();
    let config = crate::config::v2::load_v2_config(&root, Some(&root.join(".no-mistakes.yml")))
        .expect("config");
    let tsconfig = crate::codebase::ts_resolver::TsConfig {
        dir: root.clone(),
        paths_dir: root.clone(),
        ..crate::codebase::ts_resolver::TsConfig::default()
    };
    let catalog =
        crate::codebase::ts_resolver::TsConfigCatalog::forced(&root, tsconfig.clone(), None);
    let findings = run(
        PreparedRulesCheck {
            session: Arc::clone(&session),
            root: &root,
            config_path: Some(&root.join(".no-mistakes.yml")),
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
    .expect("sibling rules keep the full call plan")
    .findings;
    assert!(
        findings.iter().any(|finding| {
            finding.rule == "postgres-sql-fragment-escape"
                && finding.file == "extend.ts"
                && finding.target.as_deref() == Some("widgetColumns")
        }),
        "{findings:?}"
    );
    assert!(
        findings
            .iter()
            .all(|finding| finding.rule == "postgres-sql-fragment-escape"),
        "{findings:?}"
    );
    assert!(!session.check_consumer_ready("unconstructed-error-class"));
    assert!(!session.check_consumer_ready("forbidden-calls"));
    assert!(
        findings
            .iter()
            .all(|finding| !finding.message.contains("WidgetError")),
        "{findings:?}"
    );
    assert!(session.check_failures().iter().any(|gap| {
        gap.consumer == "postgres-sql-fragment-escape" && gap.message.contains("wide.ts")
    }));
}

fn fact_plan() -> TsFactPlan {
    TsFactPlan {
        imports: true,
        function_calls: true,
        value_flow: true,
        source: true,
        ..TsFactPlan::default()
    }
}
