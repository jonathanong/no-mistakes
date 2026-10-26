#[derive(Clone, Copy)]
enum SuppliedFactPolicy {
    FillSparse,
    RequireComplete,
}

pub(crate) struct PreparedGraphBuild<'a> {
    pub(crate) root: &'a Path,
    pub(crate) tsconfig: &'a TsConfig,
    pub(crate) tsconfig_catalog: Option<&'a crate::codebase::ts_resolver::TsConfigCatalog>,
    pub(crate) plan: GraphBuildPlan,
    pub(crate) graph_files: &'a GraphFiles,
    pub(crate) config_path: Option<&'a Path>,
    pub(crate) prepared: &'a PreparedGraphConfig,
    pub(crate) facts: Option<&'a dyn TsFactLookup>,
    pub(crate) import_resolution_cache:
        Option<&'a crate::codebase::ts_resolver::ImportResolutionCache>,
    pub(crate) dotnet_facts: Option<&'a crate::codebase::dotnet::DotnetFactMap>,
    pub(crate) swift_facts: Option<&'a crate::codebase::swift::SwiftFactMap>,
    pub(crate) visible_paths: Option<&'a crate::codebase::ts_source::VisiblePathSnapshot>,
}
