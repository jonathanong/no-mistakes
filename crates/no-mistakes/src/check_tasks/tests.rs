use super::CheckTask;
use anyhow::Result;
use no_mistakes::codebase::check_facts::CheckFactMap;
use no_mistakes::codebase::unique_exports::{self, UniqueExportFinding};
use std::path::PathBuf;

pub(crate) fn run_codebase_check(
    session: &no_mistakes::codebase::analysis_session::AnalysisSession,
    root: &std::path::Path,
    config: &no_mistakes::codebase::config::Config,
    prepared_tsconfig: &no_mistakes::codebase::ts_resolver::TsConfig,
    enabled: bool,
    facts: &CheckFactMap,
    inferred_roots: &no_mistakes::codebase::config::InferredRoots,
) -> Result<CheckTask<Vec<UniqueExportFinding>>> {
    let (findings, duration) = no_mistakes::diagnostics::measure_if_enabled(
        "analysis.codebase",
        no_mistakes::diagnostics::TimingKind::Parallel,
        || -> Result<_> {
            Ok(if enabled {
                unique_exports::analyze_project_with_prepared_facts_and_inferred_and_session(
                    root,
                    config,
                    prepared_tsconfig,
                    facts,
                    inferred_roots,
                    session,
                )?
            } else {
                Vec::new()
            })
        },
    );
    Ok(CheckTask {
        findings: findings?,
        warning: None,
        duration,
    })
}

#[test]
fn run_codebase_check_uses_explicit_tsconfig_with_shared_facts() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/unique-exports-basic/fixture");
    let config = root.join(".no-mistakes.yml");
    let files = no_mistakes::codebase::ts_source::discover_files(&root, &[]);
    let facts = no_mistakes::codebase::check_facts::collect_check_facts(
        &root,
        files,
        no_mistakes::codebase::check_facts::CheckFactPlan {
            source: true,
            symbols: true,
            ..Default::default()
        },
    );

    let prepared_tsconfig =
        no_mistakes::codebase::ts_resolver::load_tsconfig(&root.join("tsconfig.json")).unwrap();
    let loaded_config = no_mistakes::codebase::config::load_codebase_config_with_path(
        &root,
        Some(config.as_path()),
    )
    .unwrap();
    let session = no_mistakes::codebase::analysis_session::AnalysisSession::new(None);
    let results = run_codebase_check(
        &session,
        &root,
        &loaded_config,
        &prepared_tsconfig,
        true,
        &facts,
        &no_mistakes::codebase::config::InferredRoots::default(),
    )
    .unwrap();

    assert!(!results.findings.is_empty());
}
