struct SharedCheckPreparationOptions<'a> {
    config_path: Option<&'a Path>,
    tsconfig_path: Option<&'a Path>,
    deadlines: bool,
    effort: Option<no_mistakes::config::v2::Effort>,
}

struct SharedCheckContext {
    session: std::sync::Arc<no_mistakes::codebase::analysis_session::AnalysisSession>,
    root: PathBuf,
    config_path: Option<PathBuf>,
    tsconfig_path: Option<PathBuf>,
    prepared: no_mistakes::check_runner::prepared::PreparedCheckInputs,
    plan: no_mistakes::codebase::check_facts::CheckFactPlan,
    playwright_fact_plan: Option<no_mistakes::codebase::check_facts::PlaywrightFactPlan>,
    fact_files: Vec<PathBuf>,
    supplemental_call_site_files: Vec<PathBuf>,
    graph_files: Vec<PathBuf>,
    fs_files: Vec<PathBuf>,
    disable_directives: no_mistakes::codebase::rules::disable_directive_audit::PreparedDisableDirectives,
    prepared_graph: Option<no_mistakes::codebase::dependencies::graph::PreparedGraphConfig>,
    react_enabled: bool,
    queues_enabled: bool,
    unique_exports_enabled: bool,
    filesystem_rules_enabled: bool,
    graph_rules_enabled: bool,
    playwright_rules_enabled: bool,
    graph_plan: Option<no_mistakes::codebase::dependencies::graph::GraphBuildPlan>,
}
