use super::*;

pub(super) fn saved_graph(root: &Path) -> DepGraph {
    let config = TsConfig {
        dir: root.to_path_buf(),
        paths: vec![],
        paths_dir: root.to_path_buf(),
        base_url: None,
    };
    DepGraph::build_with_plan(
        root,
        &config,
        GraphBuildPlan {
            calls: true,
            effect_selectors: true,
            ..Default::default()
        },
    )
    .unwrap()
}

#[test]
fn alternative_projection_keeps_trusted_provenance_and_physical_offsets() {
    use crate::codebase::dependencies::graph::{
        KnownCallTarget, ResolvedCallAlternatives, ResolvedCallCandidate, ResolvedCallTarget,
    };
    let fixture = alternatives_fixture();
    let graph = saved_graph(fixture.path());
    let config: crate::config::v2::NoMistakesConfig = serde_yaml::from_str(
        &std::fs::read_to_string(fixture.path().join(".no-mistakes.yml")).unwrap(),
    )
    .unwrap();
    let selection = selection_from_config(&config, "database", &[]).unwrap();
    let leaf = |export: &str| ResolvedCallCandidate {
        target: KnownCallTarget::ModuleExport {
            specifier: "@vendor/database".into(),
            export_path: export.into(),
            repository_target: None,
            callable_id: None,
        },
        node: None,
    };
    // Clone real saved-graph occurrences: projection tests do not require the
    // separate replacement producer to synthesize every metadata combination.
    for callee in ["transaction", "returned"] {
        let original = graph
            .resolved_call_sites()
            .iter()
            .find(|site| site.file.ends_with("provenance.mts") && site.source_callee == callee)
            .unwrap();
        assert!(!original.typed_from.is_empty() || !original.returned_from.is_empty());
        for known in [
            vec![leaf("begin"), leaf("primary.read")],
            vec![leaf("primary.read"), leaf("begin")],
        ] {
            let mut site = original.clone();
            site.target = ResolvedCallTarget::Alternatives(std::sync::Arc::new(
                ResolvedCallAlternatives::new(known, true),
            ));
            let mut matched = Vec::new();
            visit_effect_matches(&selection, &site, &crate::fx::fx_map(), &mut |candidate| {
                matched.push(candidate)
            });
            matched.sort();
            matched.dedup();
            assert_eq!(matched.len(), 3, "{callee}: {matched:?}");
            assert!(matched
                .iter()
                .any(|candidate| candidate.callee == "primary.read"
                    && candidate.category.as_deref() == Some("read")));
            assert_eq!(
                matched
                    .iter()
                    .filter(|candidate| candidate.category.as_deref() == Some("transaction"))
                    .count(),
                2
            );
            assert_eq!(site.offset, original.offset);
            assert_eq!(site.line, original.line);
            for category in ["read", "transaction"] {
                let filtered =
                    selection_from_config(&config, "database", &[category.into()]).unwrap();
                let mut selected = Vec::new();
                visit_effect_matches(&filtered, &site, &crate::fx::fx_map(), &mut |candidate| {
                    selected.push(candidate)
                });
                assert!(selected
                    .iter()
                    .all(|candidate| candidate.category.as_deref() == Some(category)));
                assert!(!selected.is_empty());
            }
        }
    }
}

#[test]
fn degenerate_target_sets_preserve_singleton_precedence_without_inventing_unknown_effects() {
    use crate::codebase::dependencies::graph::{
        KnownCallTarget, ResolvedCallAlternatives, ResolvedCallCandidate, ResolvedCallTarget,
    };
    let fixture = alternatives_fixture();
    let graph = saved_graph(fixture.path());
    let config: crate::config::v2::NoMistakesConfig = serde_yaml::from_str(
        &std::fs::read_to_string(fixture.path().join(".no-mistakes.yml")).unwrap(),
    )
    .unwrap();
    let selection = selection_from_config(&config, "database", &[]).unwrap();
    let original = graph
        .resolved_call_sites()
        .iter()
        .find(|site| site.file.ends_with("provenance.mts") && site.source_callee == "transaction")
        .unwrap();
    assert!(!original.typed_from.is_empty());
    for (known_count, unknown, expected) in [
        (0, true, vec!["TransactionQuery"]),
        (1, false, vec!["primary.read"]),
        (2, false, vec!["primary.read"]),
        (1, true, vec!["TransactionQuery", "primary.read"]),
    ] {
        let known = (0..known_count)
            .map(|_| ResolvedCallCandidate {
                target: KnownCallTarget::ModuleExport {
                    specifier: "@vendor/database".into(),
                    export_path: "primary.read".into(),
                    repository_target: None,
                    callable_id: None,
                },
                node: None,
            })
            .collect();
        let mut site = original.clone();
        site.target = ResolvedCallTarget::Alternatives(std::sync::Arc::new(
            ResolvedCallAlternatives::new(known, unknown),
        ));
        let mut matched = Vec::new();
        visit_effect_matches(&selection, &site, &crate::fx::fx_map(), &mut |candidate| {
            matched.push(candidate)
        });
        matched.sort();
        matched.dedup();
        assert_eq!(
            matched
                .iter()
                .map(|candidate| candidate.callee.as_str())
                .collect::<Vec<_>>(),
            expected
        );
        assert_eq!(has_effect_alternatives(&site), known_count > 0 && unknown);
        assert_eq!(site.offset, original.offset);
    }
}
