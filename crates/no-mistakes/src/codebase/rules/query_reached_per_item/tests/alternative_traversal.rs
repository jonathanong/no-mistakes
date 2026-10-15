use super::*;
use crate::codebase::dependencies::graph::{GraphBuildPlan, KnownCallTargetRef};

#[test]
fn real_replacement_alternatives_keep_reached_nontransaction_evidence() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &root().with_file_name("alternatives-fixture"),
    );
    // The public rules request owns one canonical graph. The separate graph
    // request below verifies the producer contract rather than replacing it.
    let findings = crate::codebase::rules::run_check(&root, None, None).unwrap();
    assert_eq!(findings.len(), 1, "{findings:#?}");
    let finding = &findings[0];
    assert_eq!(finding.rule, RULE_ID);
    assert_eq!((finding.file.as_str(), finding.line), ("src/entry.mts", 5));
    assert_eq!(finding.import.as_deref(), Some("receiver.transaction"));
    assert!(
        finding.message.contains("queryHelper → read"),
        "{finding:#?}"
    );
    assert!(
        !finding.message.contains("transaction client only"),
        "{finding:#?}"
    );

    let tsconfig = crate::codebase::dependencies::TsConfig {
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
    let sites = graph.call_sites_in_file(&root.join("src/entry.mts"));
    let physical = sites
        .iter()
        .filter(|site| site.source_callee == "receiver.transaction")
        .collect::<Vec<_>>();
    assert_eq!(physical.len(), 1);
    let site = physical[0];
    assert_eq!(site.targets().len(), 2, "{site:#?}");
    assert!(!site.has_unknown_target(), "{site:#?}");
    assert!(site.targets().any(|candidate| matches!(
        candidate.target,
        KnownCallTargetRef::ModuleExport {
            specifier: "@vendor/database",
            export_path: "primary.transaction",
            ..
        }
    )));
    assert!(site.targets().any(|candidate| matches!(
        candidate.target,
        KnownCallTargetRef::RepositoryFunction {
            scope: "queryHelper",
            ..
        }
    ) && candidate.node.is_some()));
    assert_eq!(site.target_nodes().count(), 1);
    let source = std::fs::read_to_string(root.join("src/entry.mts")).unwrap();
    assert_eq!(
        site.offset as usize,
        source.find("receiver.transaction(item)").unwrap()
    );
    assert_eq!(site.line as usize, finding.line);
}
