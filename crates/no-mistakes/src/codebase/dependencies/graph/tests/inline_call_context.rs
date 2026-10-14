use super::*;

#[test]
fn inline_helper_retains_enclosing_context_before_later_receiver_write() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("inline-call-context"));
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
    for name in [
        "named",
        "inline",
        "control",
        "function",
        "default",
        "returned",
        "unrelated",
    ] {
        let sites = graph.call_sites_in_file(&root.join(format!("src/{name}.mts")));
        assert!(
            sites.iter().any(|site| matches!(&site.target,
                ResolvedCallTarget::ModuleExport { export_path, .. } if export_path == "primary.get"
            )),
            "{name}: {sites:#?}"
        );
    }
    for name in [
        "after",
        "captured",
        "callback",
        "bound-escape",
        "direct-escape",
        "mixed-callback",
        "other-context",
    ] {
        let sites = graph.call_sites_in_file(&root.join(format!("src/{name}.mts")));
        assert!(
            !sites.iter().any(|site| matches!(&site.target,
                ResolvedCallTarget::ModuleExport { export_path, .. } if export_path == "primary.get"
            )),
            "{name}: {sites:#?}"
        );
    }
}
