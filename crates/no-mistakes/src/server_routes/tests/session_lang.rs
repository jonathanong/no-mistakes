use std::path::PathBuf;
use std::sync::Arc;

pub(crate) fn prepared_with_empty_client_relationships(
    root: PathBuf,
    tsconfig: crate::codebase::ts_resolver::TsConfig,
    config: crate::config::v2::NoMistakesConfig,
    session: Arc<crate::codebase::analysis_session::AnalysisSession>,
) -> crate::server_routes::PreparedServerAnalysis {
    let client_relationships =
        super::super::graph::PreparedClientRelationships::new(&[], &tsconfig, &session);
    crate::server_routes::PreparedServerAnalysis {
        root,
        source_files: Arc::new(Vec::new()),
        tsconfig,
        config: Some(config),
        facts: crate::codebase::ts_source::facts::TsFactMap::default(),
        client_relationships,
        session,
    }
}

pub(crate) fn merge_language_route_facts(
    prepared: &crate::server_routes::PreparedServerAnalysis,
    facts: &mut std::collections::HashMap<PathBuf, crate::server_routes::model::FileFacts>,
) {
    super::super::lang::merge_language_route_facts(prepared, facts, None);
}
