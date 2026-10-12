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

fn configure_check_filesystem_facts(
    root: &Path,
    config: &crate::config::v2::NoMistakesConfig,
    files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
    plan: &mut crate::codebase::check_facts::CheckFactPlan,
) -> Result<()> {
    crate::codebase::rules::required_companion_imports::configure_fact_plan(
        root, config, files, sources, plan,
    )?;
    crate::codebase::rules::test_no_dependency_pins::configure_fact_plan(root, config, files, plan)
}
