struct PreparedPlaywrightView {
    settings: no_mistakes::playwright::config::Settings,
    fact_plan: no_mistakes::codebase::check_facts::PlaywrightFactPlan,
}

struct PreparedScope {
    options: AnalyzeProjectOptions,
    ordinary_calls: Option<no_mistakes::codebase::queries::call_sites::prepared::OrdinaryCallSites>,
    traversal: SharedTraversalContext,
    facts: no_mistakes::codebase::check_facts::CheckFactMap,
    check_facts: no_mistakes::codebase::check_facts::CheckFactMap,
    symbol_facts: no_mistakes::codebase::check_facts::CheckFactMap,
    import_usages: HashMap<String, no_mistakes::codebase::import_usages::PreparedImportUsages>,
    server: Option<no_mistakes::server_routes::PreparedServerAnalysis>,
    check: Option<SharedCheckContext>,
    ci_topology:
        Option<std::sync::Arc<no_mistakes::codebase::ci_workflows::PreparedWorkflowTopologyInputs>>,
    check_uses_traversal_graph: bool,
    playwright: HashMap<String, PreparedPlaywrightView>,
    queue_reports: QueueReports,
    queue_traversal_keys: std::collections::HashSet<String>,
    server_indexed_reports: ReportCache<no_mistakes::server_routes::PreparedProjectReport>,
    server_traversal_keys: std::collections::HashSet<String>,
    server_reports: ReportCache<no_mistakes::server_routes::ProjectReport>,
    playwright_analyses: ReportCache<std::sync::Arc<no_mistakes::playwright::analysis::types::Analysis>>,
    react_analyses: ReportCache<Vec<no_mistakes::react_traits::ComponentFacts>>,
}

type ReportCell<T> = std::sync::Arc<std::sync::OnceLock<Result<T, std::sync::Arc<str>>>>;
type ReportCache<T> = std::sync::Mutex<HashMap<String, ReportCell<T>>>;

struct ScopeFactPlan {
    files: Vec<PathBuf>,
    graph_files: Vec<PathBuf>,
    plan: no_mistakes::codebase::check_facts::CheckFactPlan,
    playwright: Option<no_mistakes::codebase::check_facts::PlaywrightFactPlan>,
    sources: std::sync::Arc<no_mistakes::codebase::ts_source::SourceStore>,
}

struct PreparedScopePlan {
    options: AnalyzeProjectOptions,
    ordinary_calls: Option<no_mistakes::codebase::queries::call_sites::prepared::OrdinaryCallSitesPlan>,
    root: PathBuf,
    traversal: SharedTraversalContext,
    primary: ScopeFactPlan,
    supplemental: ScopeFactPlan,
    supplemental_call_sites: ScopeFactPlan,
    configs: std::collections::HashSet<PathBuf>,
    import_usages: HashMap<String, no_mistakes::codebase::import_usages::PreparedImportUsages>,
    check: Option<SharedCheckContext>,
    ci_topology:
        Option<std::sync::Arc<no_mistakes::codebase::ci_workflows::PreparedWorkflowTopologyInputs>>,
    playwright: HashMap<String, PreparedPlaywrightView>,
    queue_traversal_keys: std::collections::HashSet<String>,
    server_traversal_keys: std::collections::HashSet<String>,
    session: std::sync::Arc<no_mistakes::codebase::analysis_session::AnalysisSession>,
}
