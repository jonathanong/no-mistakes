struct PreparedPlaywrightView {
    settings: crate::playwright::config::Settings,
    fact_plan: crate::codebase::check_facts::PlaywrightFactPlan,
}

struct PreparedScope {
    options: AnalyzeProjectOptions,
    traversal: SharedTraversalContext,
    facts: crate::codebase::check_facts::CheckFactMap,
    symbol_facts: crate::codebase::check_facts::CheckFactMap,
    import_usages: HashMap<String, crate::codebase::import_usages::PreparedImportUsages>,
    server: Option<crate::server_routes::PreparedServerAnalysis>,
    check: Option<SharedCheckContext>,
    playwright: HashMap<String, PreparedPlaywrightView>,
    queue_reports: HashMap<String, crate::queue::ProjectReport>,
    queue_indexed_reports: HashMap<String, crate::queue::PreparedProjectReport>,
    queue_traversal_keys: std::collections::HashSet<String>,
    server_indexed_reports: HashMap<String, crate::server_routes::PreparedProjectReport>,
    server_traversal_keys: std::collections::HashSet<String>,
    server_reports: HashMap<String, crate::server_routes::ProjectReport>,
    playwright_analyses: HashMap<String, crate::playwright::analysis::types::Analysis>,
    react_analyses: HashMap<String, Vec<crate::react_traits::ComponentFacts>>,
}

struct ScopeFactPlan {
    files: Vec<PathBuf>,
    graph_files: Vec<PathBuf>,
    plan: crate::codebase::check_facts::CheckFactPlan,
    playwright: Option<crate::codebase::check_facts::PlaywrightFactPlan>,
    sources: std::sync::Arc<crate::codebase::ts_source::SourceStore>,
}

struct PreparedScopePlan {
    options: AnalyzeProjectOptions,
    root: PathBuf,
    traversal: SharedTraversalContext,
    primary: ScopeFactPlan,
    supplemental: ScopeFactPlan,
    configs: std::collections::HashSet<PathBuf>,
    import_usages: HashMap<String, crate::codebase::import_usages::PreparedImportUsages>,
    check: Option<SharedCheckContext>,
    playwright: HashMap<String, PreparedPlaywrightView>,
    queue_traversal_keys: std::collections::HashSet<String>,
    server_traversal_keys: std::collections::HashSet<String>,
    session: std::sync::Arc<crate::codebase::analysis_session::AnalysisSession>,
}
