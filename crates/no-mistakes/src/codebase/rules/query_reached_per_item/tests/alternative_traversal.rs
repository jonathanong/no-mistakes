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
            effect_selectors: true,
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
    let helper_file = root.join("src/helpers.mts");
    let helper_nodes = graph.module_export_nodes("./helpers.mjs", "queryHelper");
    assert_eq!(helper_nodes.len(), 1, "{helper_nodes:#?}");
    let helper_node = &helper_nodes[0];
    let crate::codebase::dependencies::graph::NodeId::Symbol {
        file,
        symbol,
        callable_id: Some(helper_id),
        ..
    } = helper_node
    else {
        panic!("missing exact canonical helper: {helper_node:#?}");
    };
    assert_eq!(file.as_ref(), helper_file.as_path());
    assert_eq!(symbol.as_ref(), "queryHelper");
    // An imported callable preserves its module selector and repository
    // backing. Same-file Callable values use RepositoryFunction instead.
    assert!(
        site.targets().any(|candidate| matches!(
            candidate.target,
            KnownCallTargetRef::ModuleExport {
                specifier: "./helpers.mjs",
                export_path: "queryHelper",
                repository_target: Some((file, "queryHelper")),
                callable_id: Some(id),
            } if file == helper_file.as_path() && id == *helper_id
        ) && candidate.node == Some(helper_node)),
        "{site:#?}"
    );
    assert_eq!(site.target_nodes().count(), 1);
    let source = std::fs::read_to_string(root.join("src/entry.mts")).unwrap();
    assert_eq!(
        site.offset as usize,
        source.find("receiver.transaction(item)").unwrap()
    );
    assert_eq!(site.line as usize, finding.line);
}

#[test]
fn a_plain_import_replacement_adds_nontransaction_evidence_to_the_physical_call() {
    use crate::codebase::dependencies::graph::{
        KnownCallTarget, ResolvedCallAlternatives, ResolvedCallCandidate, ResolvedCallTarget,
    };
    let root = crate::codebase::ts_resolver::normalize_path(
        &root().with_file_name("plain-alternatives-fixture"),
    );
    let findings = crate::codebase::rules::run_check(&root, None, None).unwrap();
    assert_eq!(findings.len(), 1, "{findings:#?}");
    let finding = &findings[0];
    assert_eq!(
        (
            finding.file.as_str(),
            finding.line,
            finding.import.as_deref()
        ),
        ("src/entry.mts", 3, Some("receiver.transaction"))
    );
    assert!(finding.message.contains("receiver.transaction"));
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
    let mut exports = site
        .targets()
        .map(|candidate| match candidate.target {
            KnownCallTargetRef::ModuleExport {
                specifier: "@vendor/plain-alternatives",
                export_path,
                ..
            } => export_path,
            other => panic!("unexpected candidate: {other:?}"),
        })
        .collect::<Vec<_>>();
    exports.sort();
    assert_eq!(exports, ["primary.transaction", "read"]);
    assert!(!site.has_unknown_target());
    let source = std::fs::read_to_string(root.join("src/entry.mts")).unwrap();
    assert_eq!(
        site.offset as usize,
        source.find("receiver.transaction(item)").unwrap()
    );

    let config: crate::config::v2::NoMistakesConfig =
        serde_yaml::from_str(&std::fs::read_to_string(root.join(".no-mistakes.yml")).unwrap())
            .unwrap();
    let mut effect = config.effects["database"].clone();
    let mut contract = site.clone();
    contract.source_callee = "unmatchedAlias".into();
    for (unknown, expected) in [(false, None), (true, Some(false))] {
        contract.target =
            ResolvedCallTarget::Alternatives(std::sync::Arc::new(ResolvedCallAlternatives::new(
                vec![ResolvedCallCandidate {
                    target: KnownCallTarget::ModuleExport {
                        specifier: "@vendor/plain-alternatives".into(),
                        export_path: "read".into(),
                        repository_target: None,
                        callable_id: None,
                    },
                    node: None,
                }],
                unknown,
            )));
        assert_eq!(Sinks::new(&effect, &graph).match_site(&contract), expected);
    }
    let mut collision = effect.clone();
    collision.functions.clear();
    collision.transaction_functions.push("read".into());
    collision
        .targets
        .push(crate::config::v2::schema::EffectTargetConfig {
            module: "@vendor/plain-alternatives".into(),
            export: "read".into(),
            category: None,
        });
    // The plain transaction spelling is only a fallback: the explicit module
    // target still supplies nontransaction evidence for this known leaf.
    assert_eq!(
        Sinks::new(&collision, &graph).match_site(&contract),
        Some(false)
    );
    collision.targets.clear();
    assert_eq!(
        Sinks::new(&collision, &graph).match_site(&contract),
        Some(true)
    );
    let origin = crate::config::v2::schema::EffectTargetConfig {
        module: "@vendor/plain-alternatives".into(),
        export: "TransactionClient".into(),
        category: None,
    };
    contract
        .typed_from
        .push((origin.module.clone(), origin.export.clone()));
    effect.typed_executors.push(origin);
    // One known leaf plus unknown possibilities keeps trusted singleton provenance.
    assert_eq!(
        Sinks::new(&effect, &graph).match_site(&contract),
        Some(true)
    );
}
