use super::*;

#[test]
fn computed_class_phase_diagnostic() {
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
    let sites = graph.call_sites_in_file(&root.join("src/computed-class-phase.mts"));
    let matching: Vec<_> = sites
        .iter()
        .filter(|site| site.source_callee == "fn")
        .collect();
    assert_eq!(matching.len(), 3, "three physical diagnostic body calls");
    for site in matching {
        let caller = site.caller.as_deref().unwrap_or("<module>");
        let target = match &site.target {
            ResolvedCallTarget::ModuleExport { export_path, .. } => {
                serde_json::json!({"kind":"ModuleExport", "exportPath":export_path})
            }
            ResolvedCallTarget::Unknown => serde_json::json!({"kind":"Unknown"}),
            _ => serde_json::json!({"kind":"OtherKnown"}),
        };
        println!(
            "2021_CLASS_PROBE {}",
            serde_json::json!({
                "file":"src/computed-class-phase.mts", "caller":caller,
                "callee":"fn", "offset":site.offset, "target":target,
            })
        );
    }
}
