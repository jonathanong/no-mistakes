use super::*;

#[test]
fn parent_snapshot_phase_diagnostic() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture(
        "value-flow-receiver-evaluation-order",
    ));
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
    let sites = graph.call_sites_in_file(&root.join("src/repeated-parameter-phase.mts"));
    for caller in [
        "runSaved/repeatedSaved",
        "repeatedCopied",
        "setupOnce",
        "runOuter/repeatedOuter",
    ] {
        let matching: Vec<_> = sites
            .iter()
            .filter(|site| site.caller.as_deref() == Some(caller) && site.source_callee == "fn")
            .collect();
        assert_eq!(matching.len(), 1, "physical diagnostic site: {caller}");
        let site = matching[0];
        let target = match &site.target {
            ResolvedCallTarget::ModuleExport { export_path, .. } => {
                serde_json::json!({"kind":"ModuleExport", "exportPath":export_path})
            }
            ResolvedCallTarget::Unknown => serde_json::json!({"kind":"Unknown"}),
            _ => serde_json::json!({"kind":"OtherKnown"}),
        };
        println!(
            "2021_PARENT_PROBE {}",
            serde_json::json!({
                "file":"src/repeated-parameter-phase.mts", "caller":caller,
                "callee":"fn", "offset":site.offset, "target":target,
            })
        );
    }
}
