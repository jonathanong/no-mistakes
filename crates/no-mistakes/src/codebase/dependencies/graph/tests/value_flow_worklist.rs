use super::*;

#[test]
fn higher_order_chain_only_revisits_dependent_sites() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("value-flow-worklist"));
    let tsconfig = TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    let graph = DepGraph::build_with_plan(
        &root,
        &tsconfig,
        GraphBuildPlan {
            calls: true,
            ..GraphBuildPlan::default()
        },
    )
    .unwrap();
    let file = root.join("src/chain.mts");
    for index in 0..31 {
        assert!(
            graph
                .call_sites_in_file(&file)
                .iter()
                .any(|site| site.file == file
                    && site.caller.as_deref() == Some(format!("step{index}").as_str())
                    && matches!(&site.target, ResolvedCallTarget::RepositoryFunction { scope, .. }
                if scope == &format!("step{}", index + 1))),
            "missing step {index}: {:#?}",
            graph.call_sites_in_file(&file)
        );
    }
    #[cfg(feature = "test-instrumentation")]
    assert!(
        benchmark_probe_value_flow_evaluations() <= 160,
        "unrelated unresolved sites must not be rescanned with each newly resolved caller"
    );
}

#[test]
fn repeated_receiver_sites_share_one_parameter_agreement() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("value-flow-parameter-cache"));
    let tsconfig = TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    let graph = DepGraph::build_with_plan(
        &root,
        &tsconfig,
        GraphBuildPlan {
            calls: true,
            ..Default::default()
        },
    )
    .unwrap();
    let sites = graph.call_sites_in_file(&root.join("src/helper.mts"));
    assert_eq!(
        sites
            .iter()
            .filter(|site| site.source_callee == "client.get"
                && matches!(&site.target,
        ResolvedCallTarget::ModuleExport { export_path, .. } if export_path == "primary.get"))
            .count(),
        128
    );
    #[cfg(feature = "test-instrumentation")]
    assert_eq!(benchmark_probe_parameter_agreements(), 1);
}

#[test]
fn nested_mutation_alias_cycles_terminate() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("value-flow-mutation-cycle"));
    let tsconfig = TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    DepGraph::build_with_plan(
        &root,
        &tsconfig,
        GraphBuildPlan {
            calls: true,
            ..Default::default()
        },
    )
    .unwrap();
}
