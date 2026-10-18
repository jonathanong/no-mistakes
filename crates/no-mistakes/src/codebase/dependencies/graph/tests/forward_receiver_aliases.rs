use super::*;

#[test]
fn initialized_receiver_aliases_do_not_depend_on_writer_declaration_order() {
    let (root, graph) = forward_graph();
    for file in ["before", "after"] {
        assert_forward_targets(
            &graph,
            &root,
            file,
            &[
                ("value.get", None),
                ("changed.get", None),
                ("value.set", Some("primary.set")),
                ("copiedPrimary", Some("primary.get")),
                ("copiedReplica", Some("replica.get")),
                ("unrelated.get", Some("unrelated.get")),
            ],
        );
    }
}

fn forward_graph() -> (PathBuf, DepGraph) {
    let root =
        crate::codebase::ts_resolver::normalize_path(&fixture("value-flow-forward-receiver-alias"));
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
    (root, graph)
}

fn assert_forward_targets(
    graph: &DepGraph,
    root: &Path,
    file: &str,
    expected: &[(&str, Option<&str>)],
) {
    let sites = graph.call_sites_in_file(&root.join(format!("src/{file}.mts")));
    for (callee, export) in expected.iter().copied() {
        let matching: Vec<_> = sites
            .iter()
            .filter(|site| site.source_callee == callee)
            .collect();
        assert_eq!(matching.len(), 1, "{file}: {callee}: {matching:#?}");
        let site = matching[0];
        if let Some(export) = export {
            let targets: Vec<_> = site.targets().collect();
            assert_eq!(targets.len(), 1, "{file}: {site:#?}");
            assert!(
                matches!(targets[0].target,
                    KnownCallTargetRef::ModuleExport { export_path, .. } if export_path == export),
                "{file}: {site:#?}"
            );
            assert!(!site.has_unknown_target(), "{file}: {site:#?}");
        } else {
            assert!(
                site.targets().next().is_none() && site.has_unknown_target(),
                "{file}: {site:#?}"
            );
        }
    }
}

#[test]
fn forward_receiver_aliases_preserve_selected_root_snapshots_and_shadowing() {
    let (root, graph) = forward_graph();
    for (file, expected) in [
        (
            "only-before",
            vec![("value.get", None), ("changed.get", Some("replica.get"))],
        ),
        (
            "only-after",
            vec![("value.get", Some("primary.get")), ("changed.get", None)],
        ),
        (
            "shadow",
            vec![("value.get", Some("primary.get")), ("changed.get", None)],
        ),
        // Availability alone never proves that an uncalled/TDZ writer ran.
        // Retain the existing conservative policy of the declaration-after case.
        ("uncalled", vec![("value.get", None)]),
        ("uncalled-after", vec![("value.get", None)]),
        ("tdz", vec![("value.get", None)]),
    ] {
        assert_forward_targets(&graph, &root, file, &expected);
    }
}

#[test]
fn mutable_aliases_and_parameters_invalidate_the_receiver_used_at_the_write() {
    let (root, graph) = forward_graph();
    for (file, changed) in [
        ("parameter-live", "replica"),
        ("parameter-object", "replica"),
        ("captured-let", "primary"),
        ("rebound-let", "replica"),
        ("import-let", "replica"),
        ("forward-let", "replica"),
        ("forward-var", "replica"),
        ("member-swap", "replica"),
        ("loop-let", "replica"),
    ] {
        // Each case has an independent imported control that must stay Known.
        assert_forward_targets(
            &graph,
            &root,
            file,
            &[
                (&format!("{changed}.get"), None),
                ("untouched.get", Some("untouched.get")),
            ],
        );
    }
    // The direct call receives the live box after its rebind.
    assert_forward_targets(
        &graph,
        &root,
        "parameter-live",
        &[("primary.get", Some("primary.get"))],
    );
}
