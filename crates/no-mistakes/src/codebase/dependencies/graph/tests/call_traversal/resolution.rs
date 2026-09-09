use super::*;

#[test]
fn call_resolution_follows_immutable_aliases_and_reexported_defaults_only() {
    let (root, graph) = call_fixture_graph();
    let aliases = root.join("src/aliases.mts");
    let roots = graph.expand_call_roots(&[CallRoot::File(aliases.clone())]);
    let traces = graph.call_traces(&roots, CallTraversal::Direct, None);

    assert!(graph.call_traces(&[NodeId::file(&aliases)], CallTraversal::Direct, None).iter().any(
        |trace| matches!(&trace.target, NodeId::Symbol { symbol, .. } if symbol.starts_with("<anonymous:")),
    ));

    assert_eq!(
        traces
            .iter()
            .filter(|trace| has_symbol(
                &trace.target,
                &root.join("src/alias-target.mts"),
                "importedTarget"
            ))
            .count(),
        3,
        "module, anonymous callback, and nested function aliases retain canonical call edges"
    );
    for callee in ["first", "second"] {
        assert!(
            graph.resolved_call_sites().iter().any(|site| {
                site.file == aliases
                    && site.source_callee == callee
                    && matches!(
                        &site.target,
                        ResolvedCallTarget::ModuleExport {
                            repository_target: Some((file, scope)),
                            ..
                        } if file == &root.join("src/alias-target.mts") && scope == "importedTarget"
                    )
            }),
            "{callee} must retain its imported target before graph edge deduplication"
        );
    }
    assert!(graph.resolved_call_sites().iter().any(|site| {
        site.file == aliases
            && site.source_callee == "moduleAlias"
            && matches!(
                &site.target,
                ResolvedCallTarget::ModuleExport {
                    repository_target: Some((file, scope)),
                    ..
                } if file == &root.join("src/alias-target.mts") && scope == "importedTarget"
            )
    }));
    for callee in ["cycle", "cycleFromB"] {
        assert!(
            graph.resolved_call_sites().iter().any(|site| {
                site.file == aliases
                    && site.source_callee == callee
                    && matches!(
                        &site.target,
                        ResolvedCallTarget::ModuleExport {
                            repository_target: Some((file, scope)),
                            ..
                        } if file == &root.join("src/star-cycle-provider.mts") && scope == "cycle"
                    )
            }),
            "{callee} must resolve through the cyclic barrel to its concrete provider"
        );
    }
    for (callee, expected_export, expected_scope) in [
        ("targets.importedTarget", "importedTarget", "importedTarget"),
        ("targets.default", "default", "defaultTarget"),
    ] {
        let matching_sites = graph
            .resolved_call_sites()
            .iter()
            .filter(|site| site.file == aliases && site.source_callee == callee)
            .collect::<Vec<_>>();
        assert!(
            matching_sites.iter().any(|site| {
                matches!(
                    &site.target,
                    ResolvedCallTarget::ModuleExport {
                        specifier,
                        export_path,
                        repository_target: Some((file, scope)),
                        ..
                    } if specifier == "./alias-target.mts"
                        && export_path == expected_export
                        && file == &root.join("src/alias-target.mts")
                        && scope == expected_scope
                )
            }),
            "a namespace member must retain its concrete export target: {callee}: {matching_sites:#?}"
        );
    }
    assert!(graph.resolved_call_sites().iter().any(|site| {
        site.file == aliases
            && site.source_callee == "remote"
            && matches!(
                &site.target,
                ResolvedCallTarget::ModuleExport {
                    specifier,
                    export_path,
                    repository_target: None,
                    ..
                } if specifier == "unresolved-runtime-library" && export_path == "remote"
            )
    }));
    assert!(graph.resolved_call_sites().iter().any(|site| {
        site.file == aliases
            && site.source_callee == "importedExternal"
            && matches!(
                &site.target,
                ResolvedCallTarget::ModuleExport {
                    specifier,
                    export_path,
                    repository_target: None,
                    ..
                } if specifier == "vitest" && export_path == "mock"
            )
    }));
    assert!(graph.resolved_call_sites().iter().any(|site| {
        site.file == aliases
            && site.source_callee == "second"
            && matches!(
                &site.target,
                ResolvedCallTarget::ModuleExport {
                    repository_target: Some((file, scope)),
                    ..
                } if file == &root.join("src/alias-target.mts") && scope == "importedTarget"
            )
    }));
    assert!(traces.iter().any(|trace| has_symbol(
        &trace.target,
        &root.join("src/alias-target.mts"),
        "defaultTarget"
    )));
    assert!(
        !traces.iter().any(|trace| {
            has_symbol(
                &trace.target,
                &root.join("src/mixed-star-callable.mts"),
                "collision",
            )
        }),
        "a non-callable export-star collision must not select the callable branch"
    );
    assert!(
        !traces.iter().any(|trace| {
            has_symbol(
                &trace.target,
                &root.join("src/mixed-star-callable.mts"),
                "externalCollision",
            )
        }),
        "an unresolved export-star candidate must not select the visible callable branch"
    );
    for (callee, expected_scope) in [
        ("exportedAlias", "importedTarget"),
        ("defaultAlias", "defaultTarget"),
    ] {
        assert!(
            graph.resolved_call_sites().iter().any(|site| {
                site.file == aliases
                    && site.source_callee == callee
                    && matches!(
                        &site.target,
                        ResolvedCallTarget::ModuleExport {
                            repository_target: Some((file, scope)),
                            ..
                        } if file == &root.join("src/alias-target.mts") && scope == expected_scope
                    )
            }),
            "{callee} must follow an exported immutable local alias"
        );
    }
    assert!(
        graph.resolved_call_sites().iter().any(|site| {
            site.file == aliases
                && site.source_callee == "localExportAlias"
                && matches!(
                    &site.target,
                    ResolvedCallTarget::ModuleExport {
                        repository_target: Some((file, scope)),
                        ..
                    } if file == &root.join("src/exported-local-aliases.mts") && scope == "localTarget"
                )
        }),
        "an exported local alias must retain its local callable target"
    );
    assert!(graph.resolved_call_sites().iter().any(|site| {
        site.file == aliases
            && site.source_callee == "window.setTimeout"
            && matches!(site.target, ResolvedCallTarget::Global { .. })
    }));
    assert!(graph.resolved_call_sites().iter().any(|site| {
        site.file == aliases
            && site.source_callee == "externalMock"
            && matches!(
                &site.target,
                ResolvedCallTarget::ModuleExport {
                    specifier,
                    export_path,
                    repository_target: None,
                    ..
                } if specifier == "vitest" && export_path == "mock"
            )
    }));
    let same_line_calls = graph
        .resolved_call_sites()
        .iter()
        .filter(|site| {
            site.file == aliases
                && matches!(site.source_callee.as_str(), "setTimeout" | "clearTimeout")
        })
        .collect::<Vec<_>>();
    assert_eq!(same_line_calls.len(), 2);
    assert!(
        same_line_calls[0].offset < same_line_calls[1].offset,
        "same-line call sites retain ordered source offsets"
    );
    for callee in [
        "defaultThroughStar",
        "ambiguous",
        "mutable",
        "mutableDeclaration",
        "cycleA",
        "collision",
        "externalCollision",
        "window.setTimeout",
        "targets.deep.member",
        "propertyImport.call",
        "hidden",
        "missingReexport",
        "absentThroughStar",
    ] {
        assert!(
            graph.resolved_call_sites().iter().any(|site| {
                site.file == aliases
                    && site.source_callee == callee
                    && matches!(site.target, ResolvedCallTarget::Unknown)
            }),
            "{callee} must remain unknown rather than guessed"
        );
    }
}

#[test]
fn reassigned_exports_do_not_resolve_as_callable_imports() {
    let (root, graph) = call_fixture_graph();
    let consumer = root.join("src/reassigned-export-consumer.mts");

    assert!(graph.resolved_call_sites().iter().any(|site| {
        site.file == consumer
            && site.source_callee == "target"
            && site.target == ResolvedCallTarget::Unknown
    }));
}
