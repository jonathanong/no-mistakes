use super::*;
use crate::codebase::dependencies::graph::{
    KnownCallTarget, ResolvedCallAlternatives, ResolvedCallCandidate, ResolvedCallTarget,
};

fn contract_fixture() -> (
    tempfile::TempDir,
    DepGraph,
    crate::config::v2::NoMistakesConfig,
) {
    let saved =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/queries/resolved-effects");
    let fixture = crate::test_support::materialize_saved_fixture(&saved);
    let tsconfig = TsConfig {
        dir: fixture.path().to_path_buf(),
        paths: vec![],
        paths_dir: fixture.path().to_path_buf(),
        base_url: None,
    };
    let graph = DepGraph::build_with_plan(
        fixture.path(),
        &tsconfig,
        GraphBuildPlan {
            calls: true,
            effect_selectors: true,
            ..Default::default()
        },
    )
    .unwrap();
    let config = serde_yaml::from_str(
        &std::fs::read_to_string(fixture.path().join(".no-mistakes.yml")).unwrap(),
    )
    .unwrap();
    (fixture, graph, config)
}

fn module_write() -> ResolvedCallCandidate {
    ResolvedCallCandidate {
        target: KnownCallTarget::ModuleExport {
            specifier: "@vendor/effects".into(),
            export_path: "write".into(),
            repository_target: None,
            callable_id: None,
        },
        node: None,
    }
}

#[test]
fn configured_repository_alternatives_use_the_canonical_export_node() {
    let (_fixture, graph, mut config) = contract_fixture();
    let mut local = config.effects["repository"].targets[0].clone();
    local.category = Some("local".into());
    config
        .effects
        .get_mut("resolved")
        .unwrap()
        .targets
        .push(local);
    let selection = selection_from_config(&config, "resolved", &[]).unwrap();
    let graph_ref = &graph;
    let target_nodes: crate::fx::FxHashMap<_, _> = selection
        .targets
        .iter()
        .flat_map(|(module, exports)| {
            exports.values().flat_map(move |target| {
                graph_ref
                    .module_export_nodes(module, &target.export)
                    .into_iter()
                    .map(move |node| (node, target))
            })
        })
        .collect();
    assert_eq!(target_nodes.len(), 1);
    let node = target_nodes.keys().next().unwrap();
    let NodeId::Symbol { file, symbol, .. } = node else {
        panic!("saved export must be a callable node: {node:?}");
    };
    let repository = ResolvedCallCandidate {
        target: KnownCallTarget::RepositoryFunction {
            file: file.to_path_buf(),
            scope: symbol.to_string(),
        },
        node: Some(node.clone()),
    };
    let original = graph
        .resolved_call_sites()
        .iter()
        .find(|site| site.source_callee == "local")
        .unwrap();
    // This is a consumer contract over a real physical occurrence and export
    // node; the replacement producer need not emit every target combination.
    for known in [
        vec![repository.clone(), module_write()],
        vec![module_write(), repository.clone()],
    ] {
        for unknown in [false, true] {
            let mut site = original.clone();
            site.target = ResolvedCallTarget::Alternatives(std::sync::Arc::new(
                ResolvedCallAlternatives::new(known.clone(), unknown),
            ));
            let mut matched = Vec::new();
            visit_effect_matches(&selection, &site, &target_nodes, &mut |value| {
                matched.push(value)
            });
            matched.sort();
            assert_eq!(
                matched,
                [
                    EffectCallAlternative {
                        callee: "write".into(),
                        category: Some("local".into())
                    },
                    EffectCallAlternative {
                        callee: "write".into(),
                        category: Some("storage".into())
                    },
                ]
            );
            assert_eq!((site.line, site.offset), (original.line, original.offset));
        }
    }
    let mut missing_node = repository;
    missing_node.node = None;
    let mut site = original.clone();
    site.target = ResolvedCallTarget::Alternatives(std::sync::Arc::new(
        ResolvedCallAlternatives::new(vec![missing_node, module_write()], false),
    ));
    let mut matched = Vec::new();
    visit_effect_matches(&selection, &site, &target_nodes, &mut |value| {
        matched.push(value)
    });
    assert_eq!(
        matched,
        [EffectCallAlternative {
            callee: "write".into(),
            category: Some("storage".into()),
        }]
    );
}

#[test]
fn global_alternatives_match_configured_names_without_inventing_effects() {
    let (_fixture, graph, config) = contract_fixture();
    let selection = selection_from_config(&config, "mixed", &[]).unwrap();
    let original = graph
        .resolved_call_sites()
        .iter()
        .find(|site| site.source_callee == "save")
        .unwrap();
    let global = |name: &str| ResolvedCallCandidate {
        target: KnownCallTarget::Global { name: name.into() },
        node: None,
    };
    // Global RHS values are outside the replacement producer's stable-leaf
    // policy. Exercise the public typed-alternatives consumer contract directly.
    for (known, unknown, expected) in [
        (
            vec![global("write"), module_write()],
            false,
            vec![None, Some("storage")],
        ),
        (
            vec![module_write(), global("write")],
            true,
            vec![None, Some("storage")],
        ),
        (
            vec![global("unconfigured"), module_write()],
            false,
            vec![Some("storage")],
        ),
        (vec![global("write")], true, vec![None]),
        // Singleton spelling is handled by the existing syntactic projection.
        (vec![global("write")], false, vec![]),
    ] {
        let mut site = original.clone();
        site.target = ResolvedCallTarget::Alternatives(std::sync::Arc::new(
            ResolvedCallAlternatives::new(known, unknown),
        ));
        let mut matched = Vec::new();
        visit_effect_matches(&selection, &site, &crate::fx::fx_map(), &mut |value| {
            matched.push(value)
        });
        matched.sort();
        assert!(matched.iter().all(|value| value.callee == "write"));
        assert_eq!(
            matched
                .iter()
                .map(|value| value.category.as_deref())
                .collect::<Vec<_>>(),
            expected
        );
        assert_eq!((site.line, site.offset), (original.line, original.offset));
    }
}
