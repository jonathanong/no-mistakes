use super::*;

fn with_fixture(
    run: impl FnOnce(&GraphEdgeBuildInputs<'_>, &dyn TsFactLookup, &dyn ImportResolution),
) {
    use crate::codebase::ts_source::facts::{TsFactPlan, collect_ts_facts};
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/value-flow-replacement-scale-16/fixture");
    let root = crate::codebase::ts_resolver::normalize_path(&root);
    let paths = vec![root.join("src/entry.mts")];
    let facts = collect_ts_facts(
        &paths,
        TsFactPlan {
            imports: true,
            function_calls: true,
            value_flow: true,
            ..Default::default()
        },
    );
    let files = GraphFiles::from_files(paths);
    let config = TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    let resolver = crate::codebase::ts_resolver::ImportResolver::new(&config);
    let inputs = GraphEdgeBuildInputs {
        root: &root,
        tsconfig: &config,
        tsconfig_catalog: None,
        plan: GraphBuildPlan {
            imports: true,
            calls: true,
            ..Default::default()
        },
        workspace: None,
        graph_files: &files,
        config_options: None,
        playwright_settings: &[],
        config_path: None,
        dotnet_facts: None,
        swift_facts: None,
        import_resolution_cache: None,
        visible_paths: None,
        workflow_documents: None,
        interner: std::sync::Arc::new(PathInterner::new()),
    };
    run(&inputs, &facts, &resolver);
}

#[test]
fn scalar_refinement_keeps_construction_and_missing_fact_boundaries() {
    with_fixture(|inputs, facts, resolver| {
        let indexes = CallableResolutionIndexes::default();
        let pass =
            collect_call_edges_for_core(inputs, facts, resolver, &mut fx_map(), &mut fx_map());
        let base = pass.sites.first().unwrap();
        let incoming = fx_map();
        let phases = execution_phases::ModuleWriterPhases::default();
        for construct in [false, true] {
            let mut site = base.clone();
            if construct {
                site.invocation = InvocationKind::Construct;
            } else {
                site.file = inputs.root.join("missing-facts.mts");
            }
            let (actual, used) = refine_site(
                inputs,
                facts,
                resolver,
                &indexes,
                &site,
                ValueInputs {
                    module_writer_phases: &phases,
                    cache: &pass.parameter_cache,
                    incoming: &incoming,
                    supers: &pass.super_constructors,
                    replacements: None,
                },
                true,
            );
            assert_eq!(actual, site);
            assert!(used.is_empty());
        }
    });
}

#[test]
fn worklist_replaces_stale_callable_incoming_without_losing_physical_calls() {
    with_fixture(|inputs, facts, resolver| {
        let indexes = CallableResolutionIndexes::default();
        let mut pass =
            collect_call_edges_for_core(inputs, facts, resolver, &mut fx_map(), &mut fx_map());
        assert_eq!(pass.sites.len(), 16);
        let expected = pass.sites.clone();
        // A stale known endpoint must be retired, not retained alongside the new one.
        for site in &mut pass.sites {
            site.target = ResolvedCallTarget::RepositoryFunction {
                file: site.file.clone(),
                scope: "stale".into(),
            };
            site.target_node = Some(NodeId::callable(&site.file, "stale", CallableId(u32::MAX)));
        }
        refine_value_calls(inputs, facts, resolver, &indexes, &mut pass);
        assert_eq!(pass.sites, expected);
    });
}
