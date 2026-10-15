use super::*;
use crate::codebase::dependencies::graph::{
    GraphBuildPlan, KnownCallTarget, ResolvedCallAlternatives, ResolvedCallCandidate,
    ResolvedCallTarget,
};

#[test]
fn mixed_alternatives_override_transaction_spelling_in_either_order() {
    let root = root();
    let ts = crate::codebase::dependencies::TsConfig {
        dir: root.clone(),
        paths: Vec::new(),
        paths_dir: root.clone(),
        base_url: None,
    };
    let graph = DepGraph::build_with_plan(
        &root,
        &ts,
        GraphBuildPlan {
            calls: true,
            ..Default::default()
        },
    )
    .unwrap();
    let mut site = graph
        .resolved_call_sites()
        .first()
        .expect("saved fixture call")
        .clone();
    site.source_callee = "primary.transaction".into();
    site.returned_from.clear();
    site.typed_from.clear();
    site.returned_member_from.clear();
    site.target_node = None;
    let effect = EffectKindConfig {
        functions: vec!["query".into()],
        transaction_functions: vec!["primary.transaction".into()],
        ..Default::default()
    };
    let sinks = Sinks::new(&effect, &graph);
    let leaf = |name: &str| ResolvedCallCandidate {
        target: KnownCallTarget::ModuleExport {
            specifier: "@vendor/database".into(),
            export_path: name.into(),
            repository_target: None,
            callable_id: None,
        },
        node: None,
    };
    for provenance in 0..3 {
        let mut effect = effect.clone();
        site.typed_from.clear();
        site.returned_from.clear();
        let origin = crate::config::v2::schema::EffectTargetConfig {
            module: "@vendor/database".into(),
            export: "TransactionClient".into(),
            category: None,
        };
        if provenance == 1 {
            site.typed_from
                .push((origin.module.clone(), origin.export.clone()));
            effect.typed_executors.push(origin);
        } else if provenance == 2 {
            site.returned_from
                .push((origin.module.clone(), origin.export.clone()));
            effect.returned_executors.push(origin);
        }
        let sinks = Sinks::new(&effect, &graph);
        for has_unknown in [false, true] {
            for known in [
                vec![leaf("primary.transaction"), leaf("replica.query")],
                vec![leaf("replica.query"), leaf("primary.transaction")],
            ] {
                site.target = ResolvedCallTarget::Alternatives(std::sync::Arc::new(
                    ResolvedCallAlternatives::new(known, has_unknown),
                ));
                assert_eq!(sinks.match_site(&site), Some(false));
                assert_eq!(site.targets().len(), 2);
                assert_eq!(site.has_unknown_target(), has_unknown);
            }
            site.target = ResolvedCallTarget::Alternatives(std::sync::Arc::new(
                ResolvedCallAlternatives::new(vec![leaf("primary.transaction")], has_unknown),
            ));
            // Unknown candidates do not invent query evidence.
            assert_eq!(sinks.match_site(&site), Some(true));
            site.target = ResolvedCallTarget::Alternatives(std::sync::Arc::new(
                ResolvedCallAlternatives::new(vec![leaf("replica.query")], has_unknown),
            ));
            // A singleton retains historical named transaction precedence,
            // even when its physical call also has an unknown alternative.
            assert_eq!(sinks.match_site(&site), Some(true));
        }
    }
    let node = graph
        .resolved_call_sites()
        .iter()
        .find_map(|site| site.target_node.clone())
        .expect("saved fixture contains repository calls");
    let mut transaction = leaf("primary.transaction");
    transaction.node = Some(node.clone());
    let mut query = leaf("replica.query");
    query.node = Some(node.clone());
    site.target = ResolvedCallTarget::Alternatives(std::sync::Arc::new(
        ResolvedCallAlternatives::new(vec![query, transaction], true),
    ));
    assert_eq!(site.targets().len(), 2);
    assert_eq!(site.target_nodes().collect::<Vec<_>>(), vec![&node]);
    assert_eq!(sinks.match_site(&site), Some(false));
}

#[test]
fn direct_transaction_and_reached_query_keep_nontransaction_evidence() {
    let direct = || Reach {
        path: vec!["transaction".into()],
        transaction: true,
    };
    let helper = || Reach {
        path: vec!["helper".into(), "query".into()],
        transaction: false,
    };
    for paths in [vec![direct(), helper()], vec![helper(), direct()]] {
        let reached = paths.into_iter().reduce(prefer_nontransaction).unwrap();
        assert!(!reached.transaction);
        assert_eq!(reached.path, ["helper", "query"]);
    }
}
