struct PreparedPlaywrightView {
    settings: crate::playwright::config::Settings,
    fact_plan: crate::codebase::check_facts::PlaywrightFactPlan,
}

struct PreparedScope {
    options: AnalyzeProjectOptions,
    traversal: SharedTraversalContext,
    facts: crate::codebase::check_facts::CheckFactMap,
    check_facts: crate::codebase::check_facts::CheckFactMap,
    symbol_facts: crate::codebase::check_facts::CheckFactMap,
    import_usages: HashMap<String, crate::codebase::import_usages::PreparedImportUsages>,
    server: Option<crate::server_routes::PreparedServerAnalysis>,
    check: Option<SharedCheckContext>,
    check_uses_traversal_graph: bool,
    playwright: HashMap<String, PreparedPlaywrightView>,
    queue_reports: ReportCache<crate::queue::ProjectReport>,
    queue_indexed_reports: ReportCache<crate::queue::PreparedProjectReport>,
    queue_traversal_keys: std::collections::HashSet<String>,
    server_indexed_reports: ReportCache<crate::server_routes::PreparedProjectReport>,
    server_traversal_keys: std::collections::HashSet<String>,
    server_reports: ReportCache<crate::server_routes::ProjectReport>,
    playwright_analyses:
        ReportCache<std::sync::Arc<crate::playwright::analysis::types::Analysis>>,
    react_analyses: ReportCache<Vec<crate::react_traits::ComponentFacts>>,
}

type ReportCell<T> =
    std::sync::Arc<std::sync::OnceLock<Result<T, std::sync::Arc<str>>>>;
type ReportCache<T> = std::sync::Mutex<HashMap<String, ReportCell<T>>>;

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
    supplemental_call_sites: ScopeFactPlan,
    configs: std::collections::HashSet<PathBuf>,
    import_usages: HashMap<String, crate::codebase::import_usages::PreparedImportUsages>,
    check: Option<SharedCheckContext>,
    playwright: HashMap<String, PreparedPlaywrightView>,
    queue_traversal_keys: std::collections::HashSet<String>,
    server_traversal_keys: std::collections::HashSet<String>,
    session: std::sync::Arc<crate::codebase::analysis_session::AnalysisSession>,
}
