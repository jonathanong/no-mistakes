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
fn projected_receiver_sites_share_one_parameter_agreement() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("value-flow-projected-cache"));
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
    let sites = graph.call_sites_in_file(&root.join("src/projected.mts"));
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

#[cfg(feature = "test-instrumentation")]
#[test]
fn fan_in_origins_remain_shared_across_receiver_fan_out() {
    let mut counts = Vec::new();
    let mut dependencies = Vec::new();
    for (name, receivers) in [
        ("value-flow-origin-small", 8),
        ("value-flow-origin-large", 128),
    ] {
        let root = crate::codebase::ts_resolver::normalize_path(&fixture(name));
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
        let calls = graph.call_sites_in_file(&root.join("src/entry.mts"));
        let sinks = calls
            .iter()
            .filter(|call| call.source_callee == "client.get")
            .collect::<Vec<_>>();
        assert_eq!(sinks.len(), receivers);
        assert!(sinks
            .iter()
            .all(|call| matches!(call.target, ResolvedCallTarget::Unknown)));
        counts.push(benchmark_probe_origin_nodes());
        dependencies.push(benchmark_probe_parameter_dependency_links());
    }
    assert!(
        dependencies[0] >= 128,
        "the fan-in must register its direct dependency edges: {dependencies:?}"
    );
    assert!(
        dependencies[1] <= dependencies[0] + 4 * (128 - 8),
        "fan-out must not retain transitive dependency sets per helper: {dependencies:?}"
    );
    assert!(
        counts[0] >= 128,
        "the fixture must collect its fan-in origins: {counts:?}"
    );
    assert!(
        counts[1] <= counts[0] + 4 * (128 - 8),
        "fan-out must add shared links, not clone 128 origins per receiver: {counts:?}"
    );
}

#[test]
fn repeated_module_snapshot_reads_share_the_closed_writer_proof() {
    let root =
        crate::codebase::ts_resolver::normalize_path(&fixture("value-flow-module-snapshot-work"));
    let config = TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    let graph = DepGraph::build_with_plan(
        &root,
        &config,
        GraphBuildPlan {
            calls: true,
            ..Default::default()
        },
    )
    .unwrap();
    let sites = graph.call_sites_in_file(&root.join("src/saved.mts"));
    assert_eq!(
        sites
            .iter()
            .filter(|site| site.source_callee == "saved"
                && matches!(&site.target, ResolvedCallTarget::ModuleExport { export_path, .. }
            if export_path == "client.get"))
            .count(),
        128
    );
    // Count real chain steps: equal targets alone would allow 128 replays.
    #[cfg(feature = "test-instrumentation")]
    assert!(
        benchmark_probe_execution_phase_steps() <= 4,
        "snapshot queries replayed the same closed writer chain: {}",
        benchmark_probe_execution_phase_steps()
    );
}

#[test]
fn callable_value_use_facts_exclude_bare_callees_and_stay_cold_without_demand() {
    for (file, expected) in [
        ("callable-uses-cold.mts", None),
        ("callable-uses-values.mts", Some("stored")),
        ("callable-uses-jsx.tsx", Some("Stored")),
    ] {
        let source = std::fs::read_to_string(
            fixture("value-flow-module-snapshot-work")
                .join("src")
                .join(file),
        )
        .unwrap();
        let allocator = oxc_allocator::Allocator::default();
        let parsed = oxc_parser::Parser::new(
            &allocator,
            &source,
            oxc_span::SourceType::from_path(file).unwrap(),
        )
        .parse();
        assert!(parsed.diagnostics.is_empty(), "{file}");
        for enabled in [false, true] {
            let facts = crate::codebase::dependencies::extract::extract_import_facts_from_program_with_source_and_demands(
                &parsed.program, &source, false, false, enabled,
            );
            let uses = facts.value_flow.callable_value_uses.as_deref();
            assert_eq!(
                uses.is_some(),
                enabled && expected.is_some(),
                "{file} demand={enabled}"
            );
            if let Some(uses) = uses {
                assert_eq!(uses.len(), 1, "shadowed parameter escaped into facts");
                assert_eq!(uses[&0].len(), 1);
                assert!(uses[&0].contains(expected.unwrap()));
            }
        }
    }
}
