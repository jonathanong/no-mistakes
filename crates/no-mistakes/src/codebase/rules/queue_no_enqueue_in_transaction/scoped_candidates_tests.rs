use super::*;
use crate::codebase::dependencies::graph::{DepGraph, GraphBuildPlan};
use crate::codebase::dependencies::TsConfig;
use crate::codebase::ts_resolver::normalize_path;

#[test]
fn repeated_receiver_replacement_produces_known_only_ambiguous_candidates() {
    let root = normalize_path(
        &crate::test_support::rule_fixture_root("queue-no-enqueue-in-transaction")
            .join("scoped-handles"),
    );
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
    let sites = graph.call_sites_in_file(&root.join("src/ambiguous.ts"));
    for callee in ["queueClient.send", "safeClient.send"] {
        let site = sites
            .iter()
            .find(|site| site.source_callee == callee)
            .unwrap_or_else(|| panic!("missing call site {callee}"));
        assert!(is_ambiguous(site), "{callee}: {site:?}");
        assert!(!site.has_unknown_target(), "{callee}: {site:?}");
        assert!(!any_global_callback(site), "{callee}: {site:?}");
        assert_eq!(site.targets().len(), 2, "{callee}: {site:?}");
    }
}
