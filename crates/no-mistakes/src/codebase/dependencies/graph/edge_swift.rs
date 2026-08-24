struct SwiftRouteDefInputs<'a> {
    root: &'a Path,
    tsconfig: &'a TsConfig,
    tsconfig_catalog: Option<&'a crate::codebase::ts_resolver::TsConfigCatalog>,
    all_files: &'a [PathBuf],
    config_options: &'a GraphConfigOptions,
    ts_facts: Option<&'a dyn TsFactLookup>,
    session: &'a crate::codebase::analysis_session::AnalysisSession,
}

struct SwiftEdgeInputs<'a> {
    root: &'a Path,
    tsconfig: &'a TsConfig,
    tsconfig_catalog: Option<&'a crate::codebase::ts_resolver::TsConfigCatalog>,
    all_files: &'a [PathBuf],
    config_options: Option<&'a GraphConfigOptions>,
    ts_facts: Option<&'a dyn TsFactLookup>,
    prepared_facts: Option<&'a crate::codebase::swift::SwiftFactMap>,
    sources: Option<&'a crate::codebase::ts_source::SourceStore>,
    session: &'a crate::codebase::analysis_session::AnalysisSession,
}

fn collect_swift_edges_with_facts(
    inputs: SwiftEdgeInputs<'_>,
    interner: &PathInterner,
) -> Vec<Edge> {
    let Some(config_options) = inputs.config_options else {
        return Vec::new();
    };
    if config_options.swift_packages.is_empty() {
        return Vec::new();
    }
    let owned_facts = inputs.prepared_facts.is_none().then(|| {
        crate::codebase::swift::collect_swift_facts_with_sources(
            inputs.root,
            inputs.all_files,
            &config_options.swift_packages,
            inputs.sources,
        )
    });
    let facts = inputs
        .prepared_facts
        .or(owned_facts.as_ref())
        .expect("Swift facts are prepared or collected");
    if facts.files.is_empty() {
        return Vec::new();
    }

    let mut edges = Vec::new();
    collect_swift_import_edges(facts, &mut edges, interner);
    collect_swift_reference_edges(facts, &mut edges, interner);
    collect_swift_package_edges(facts, &mut edges, interner);
    collect_swift_manifest_edges(facts, inputs.all_files, &mut edges, interner);
    collect_swift_http_edges(
        SwiftRouteDefInputs {
            root: inputs.root,
            tsconfig: inputs.tsconfig,
            tsconfig_catalog: inputs.tsconfig_catalog,
            all_files: inputs.all_files,
            config_options,
            ts_facts: inputs.ts_facts,
            session: inputs.session,
        },
        facts,
        &mut edges,
        interner,
    );
    edges
}
