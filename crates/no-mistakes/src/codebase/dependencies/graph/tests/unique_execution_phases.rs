use super::*;

#[test]
fn unique_caller_phase_work_is_shared_across_repeated_parameter_reads() {
    #[cfg(feature = "test-instrumentation")]
    let mut counts = Vec::new();
    for (name, reads) in [
        ("value-flow-unique-phase-small", 16),
        ("value-flow-unique-phase-large", 128),
    ] {
        let root = crate::codebase::ts_resolver::normalize_path(&fixture(name));
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
        let sites = graph.call_sites_in_file(&root.join("src/entry.mts"));
        let calls: Vec<_> = sites
            .iter()
            .filter(|site| site.caller.as_deref() == Some("consume") && site.source_callee == "fn")
            .collect();
        assert_eq!(calls.len(), reads);
        assert!(calls.iter().all(|site| matches!(&site.target, ResolvedCallTarget::ModuleExport { export_path, .. } if export_path == "sharedPhase.get")), "{calls:?}");
        #[cfg(feature = "test-instrumentation")]
        {
            let agreements = benchmark_probe_parameter_agreements();
            let steps = benchmark_probe_execution_phase_steps();
            assert!(agreements <= 4, "{name}: {agreements} agreements");
            assert!(
                steps > 0 && steps <= 32 * 3 * 4,
                "{name}: {steps} indexed phase steps"
            );
            counts.push((agreements, steps));
        }
    }
    #[cfg(feature = "test-instrumentation")]
    assert_eq!(counts[0], counts[1], "per-read phase work must stay shared");
}
