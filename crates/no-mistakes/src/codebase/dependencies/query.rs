use anyhow::Result;
use std::marker::PhantomData;
use std::path::Path;

use super::graph::{DepGraph, GraphBuildPlan, GraphFiles, PreparedGraphBuild};
use crate::codebase::analysis_session::{AnalysisSession, PathInterner};
use crate::codebase::ts_source::facts::{TsFactContext, TsFactMap, TsFactPlan};

pub(crate) struct QueryGraphRequest<'a, Selection, Select, Validate, Plan, Configure> {
    pub(crate) root: &'a Path,
    pub(crate) config_path: Option<&'a Path>,
    pub(crate) tsconfig_path: Option<&'a Path>,
    pub(crate) explicit_root: &'a Path,
    pub(crate) config_before_facts: bool,
    pub(crate) fact_plan: TsFactPlan,
    pub(crate) select: Select,
    pub(crate) validate_root: Validate,
    pub(crate) graph_plan: Plan,
    pub(crate) configure_context: Configure,
    pub(crate) selection: PhantomData<fn() -> Selection>,
}

pub(crate) struct PreparedQueryGraph<Selection> {
    pub(crate) selection: Selection,
    pub(crate) graph: DepGraph,
    pub(crate) facts: TsFactMap,
    pub(crate) interner: std::sync::Arc<PathInterner>,
}

/// Build the shared runtime import graph used by file-rooted TS queries.
/// Callers supply the complete fact demand and configure query-specific facts
/// before collection; the session remains the owner of sources and inventory.
pub(crate) fn prepare_query_graph<Selection, Select, Validate, Plan, Configure>(
    request: QueryGraphRequest<'_, Selection, Select, Validate, Plan, Configure>,
) -> Result<PreparedQueryGraph<Selection>>
where
    Select: FnOnce(Option<&crate::config::v2::schema::NoMistakesConfig>) -> Result<Selection>,
    Validate: FnOnce(&Selection, &mut GraphFiles) -> Result<()>,
    Plan: FnOnce(&Selection, &GraphFiles, &TsFactMap) -> GraphBuildPlan,
    Configure: FnOnce(&Selection, &mut TsFactContext),
{
    let QueryGraphRequest {
        root,
        config_path,
        tsconfig_path,
        explicit_root,
        config_before_facts,
        fact_plan,
        select,
        validate_root,
        graph_plan,
        configure_context,
        selection: _,
    } = request;
    let session = AnalysisSession::new(crate::diagnostics::current());
    let dataset = session.dataset(root);
    let visible_paths = dataset.visible_paths_arc();
    let root_visible_paths = dataset.paths_for(root);
    let sources = dataset.sources_for(root);
    let mut graph_files = GraphFiles::from_files_with_resource_candidates(
        crate::codebase::ts_source::discover_files_from_visible(root, &[], &root_visible_paths),
        visible_paths.tracked_paths_for(root).as_ref().clone(),
    );
    let mut config = if config_before_facts {
        Some(dataset.config(config_path)?)
    } else {
        None
    };
    let selection = select(config.as_deref())?;
    validate_root(&selection, &mut graph_files)?;
    graph_files.add_explicit_root(explicit_root);
    let explicit_tsconfig = tsconfig_path;
    let (tsconfig, tsconfig_catalog) = match explicit_tsconfig {
        None => {
            let workspace = dataset.workspace();
            let catalog = crate::codebase::ts_resolver::TsConfigCatalog::from_visible_and_sources_with_workspace(
                root,
                std::slice::from_ref(&root.to_path_buf()),
                &root_visible_paths,
                &sources,
                &workspace,
            );
            let config = catalog.config_for(explicit_root).clone();
            (config, catalog)
        }
        Some(path) => {
            let config = (*dataset.tsconfig(Some(path))?).clone();
            let path = if path.is_absolute() {
                path.to_path_buf()
            } else {
                root.join(path)
            };
            let catalog = crate::codebase::ts_resolver::TsConfigCatalog::forced(
                root,
                config.clone(),
                Some(crate::codebase::ts_resolver::normalize_path(&path)),
            );
            (config, catalog)
        }
    };

    let mut context = TsFactContext::new(root);
    context.set_visible_file_set(graph_files.visible_path_set());
    configure_context(&selection, &mut context);
    let facts =
        crate::codebase::ts_source::facts::collect_ts_facts_with_context_sources_and_session(
            &session,
            graph_files.indexable(),
            fact_plan,
            &context,
            &sources,
        );
    crate::invocation::check_timeout()?;
    let plan = graph_plan(&selection, &graph_files, &facts);

    if config.is_none() {
        config = Some(dataset.config(config_path)?);
    }
    let config = config.expect("query config is loaded before graph preparation");

    let codebase_config =
        crate::codebase::config::config_from_loaded_v2(root, config_path, &config);
    let prepared_graph = crate::codebase::dependencies::graph::prepare_graph_config(
        root,
        plan,
        &codebase_config,
        &config,
        &visible_paths,
    )?;
    let interner = session.interner_arc();
    let graph = DepGraph::build_with_plan_files_prepared_config_facts_resolution_cache_and_session(
        PreparedGraphBuild {
            root,
            tsconfig: &tsconfig,
            tsconfig_catalog: Some(&tsconfig_catalog),
            plan,
            graph_files: &graph_files,
            config_path,
            prepared: &prepared_graph,
            facts: Some(&facts),
            import_resolution_cache: None,
            dotnet_facts: None,
            swift_facts: None,
            visible_paths: Some(&visible_paths),
        },
        session,
    )?;
    Ok(PreparedQueryGraph {
        selection,
        graph,
        facts,
        interner,
    })
}
