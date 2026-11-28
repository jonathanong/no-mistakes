use crate::codebase::{
    package_analysis::{self, Hooks},
    rules, workspaces,
};

ctor::declarative::ctor! {
    // SAFETY: The constructor only installs an immutable table of function pointers.
    #[ctor(unsafe)]
    fn register_package_analysis() {
        package_analysis::register(&PACKAGE_HOOKS);
    }
}

static PACKAGE_HOOKS: Hooks = Hooks {
    load_workspace_from_files: workspaces::load_indexed_from_files,
    load_workspace_with_sources: workspaces::load_indexed_with_sources,
    manifest_edges: workspaces::edge_package_manifest::collect_workspace_manifest_edges,
    production_import_files: rules::production_dependency_declarations::required_import_fact_files,
};
