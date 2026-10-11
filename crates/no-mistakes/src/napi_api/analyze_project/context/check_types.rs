struct SharedCheckPreparationOptions<'a> {
    config_path: Option<&'a Path>,
    tsconfig_path: Option<&'a Path>,
    deadlines: bool,
}

struct SharedCheckContext {
    root: PathBuf,
    config_path: Option<PathBuf>,
    tsconfig_path: Option<PathBuf>,
    prepared: crate::check_runner::prepared::PreparedCheckInputs,
    plan: crate::codebase::check_facts::CheckFactPlan,
    playwright_fact_plan: Option<crate::codebase::check_facts::PlaywrightFactPlan>,
    fact_files: Vec<PathBuf>,
    supplemental_call_site_files: Vec<PathBuf>,
    graph_files: Vec<PathBuf>,
    fs_files: Vec<PathBuf>,
    prepared_graph: Option<crate::codebase::dependencies::graph::PreparedGraphConfig>,
    react_enabled: bool,
    queues_enabled: bool,
    unique_exports_enabled: bool,
    filesystem_rules_enabled: bool,
    graph_rules_enabled: bool,
    playwright_rules_enabled: bool,
    graph_plan: Option<crate::codebase::dependencies::graph::GraphBuildPlan>,
}

