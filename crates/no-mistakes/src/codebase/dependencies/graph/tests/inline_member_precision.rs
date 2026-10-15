use super::*;

fn graph(name: &str) -> (std::path::PathBuf, DepGraph) {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture(name));
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

#[test]
fn inline_receivers_validate_the_member_read_without_losing_escape_guards() {
    let (root, graph) = graph("inline-member-precision");
    for (file, callee, expected) in [
        ("sibling", "client.get", Some("primary.get")),
        ("matching", "client.get", None),
        ("dynamic", "client.get", None),
        ("nested-sibling", "options.client.get", Some("primary.get")),
        ("nested-matching", "options.client.get", None),
        ("default-sibling", "client.get", Some("primary.get")),
        ("default-matching", "client.get", None),
        ("copied-method", "method", Some("primary.get")),
        ("static-computed", "client.get", Some("primary.get")),
        ("default-object", "options.client.get", Some("primary.get")),
        ("default-object-matching", "options.client.get", None),
        // Transparent aliases must retain the final member projection.
        ("alias-named", "alias.get", Some("primary.get")),
        ("alias-inline", "alias.get", Some("primary.get")),
        ("alias-matching", "alias.get", None),
        ("alias-dynamic", "alias.get", None),
        ("alias-chain", "alias.get", Some("primary.get")),
        (
            "alias-nested",
            "alias.nested.get",
            Some("primary.nested.get"),
        ),
        (
            "alias-nested-sibling",
            "alias.nested.get",
            Some("primary.nested.get"),
        ),
        (
            "alias-default-nested-sibling",
            "alias.get",
            Some("primary.nested.get"),
        ),
        ("alias-default-nested-matching", "alias.get", None),
        ("alias-live", "alias.get", None),
        ("alias-early-live", "alias.get", None),
        ("alias-ancestor-write", "alias.nested.get", None),
        ("alias-default-ancestor-write", "alias.get", None),
        ("alias-supplied-live", "alias.get", None),
        ("alias-supplied-snapshot", "get", Some("primary.get")),
        ("alias-supplied-nested-live", "nested.get", None),
        (
            "alias-supplied-nested-sibling",
            "nested.get",
            Some("primary.nested.get"),
        ),
        (
            "alias-supplied-nested-snapshot",
            "get",
            Some("primary.nested.get"),
        ),
        ("alias-supplied-captured", "alias.get", None),
        ("alias-supplied-async", "alias.get", None),
        ("alias-supplied-conflicting", "alias.get", None),
        ("alias-supplied-early-live", "alias.get", None),
        ("alias-propagated", "alias.get", None),
        ("alias-unknown-input", "alias.get", None),
        ("alias-reexport", "alias.get", Some("primary.get")),
        ("alias-snapshot", "get", Some("primary.get")),
        // Copying the container freezes its identity, not its later members.
        ("alias-nested-live", "nested.get", None),
        ("alias-nested-call-live", "nested.get", None),
        ("alias-nested-apply-live", "nested.get", None),
        // Assert the bound invocation, whose syntactic callee is anonymous.
        ("alias-nested-bind-live", "<unknown>", None),
        ("alias-snapshot-call", "get", Some("primary.get")),
        ("alias-snapshot-apply", "get", Some("primary.get")),
        ("alias-snapshot-bind", "<unknown>", Some("primary.get")),
        ("alias-inline-snapshot", "get", Some("primary.get")),
        ("inline-snapshot-after-call", "get", Some("primary.get")),
        // A single inline acquisition precedes the owned body write.
        ("alias-inline-snapshot-owned", "get", Some("primary.get")),
        ("inline-snapshot-owned", "get", Some("primary.get")),
        ("default-snapshot-owned", "get", Some("primary.get")),
        ("default-object-snapshot-owned", "get", Some("primary.get")),
        ("default-wrapped-snapshot-owned", "get", Some("primary.get")),
        (
            "default-restored-snapshot-owned",
            "get",
            Some("primary.get"),
        ),
        ("default-deferred-arrow", "get", None),
        ("default-deferred-function", "get", None),
        ("default-deferred-method", "get", None),
        ("default-deferred-field", "get", None),
        ("default-deferred-accessor", "get", None),
        ("default-deferred-nested", "get", None),
        ("default-snapshot-write-first", "get", None),
        ("default-live-receiver", "alias.get", None),
        ("default-snapshot-async", "get", None),
        ("default-snapshot-captured", "get", Some("primary.get")),
        ("default-receiver-captured", "alias.get", None),
        ("inline-snapshot-owned-write-first", "get", None),
        // Repeated execution can acquire the replacement on a later pass.
        ("inline-snapshot-repeated-for", "get", None),
        ("inline-snapshot-repeated-header", "get", None),
        ("inline-snapshot-repeated-function", "get", None),
        ("inline-snapshot-repeated-while", "get", None),
        ("inline-snapshot-repeated-do", "get", None),
        ("inline-snapshot-repeated-field", "get", None),
        ("inline-snapshot-repeated-nested-field", "get", None),
        ("inline-snapshot-repeated-accessor", "get", None),
        ("alias-inline-live", "alias.get", None),
        ("alias-captured", "alias.get", None),
        ("alias-async", "alias.get", None),
        ("snapshot-named", "get", Some("primary.get")),
        ("captured", "client.get", None),
        ("async", "client.get", None),
    ] {
        let sites = graph.call_sites_in_file(&root.join(format!("src/{file}.mts")));
        let reads = sites
            .iter()
            .filter(|site| site.source_callee == callee)
            .collect::<Vec<_>>();
        assert!(!reads.is_empty(), "{file} {callee}: {sites:?}");
        for site in reads {
            assert_eq!(
                match &site.target {
                    ResolvedCallTarget::ModuleExport { export_path, .. } =>
                        Some(export_path.as_str()),
                    _ => None,
                },
                expected,
                "{file} {callee}: {site:?}"
            );
        }
    }
}

#[test]
fn mixed_member_reads_share_the_same_two_bootstrap_agreements() {
    for (fixture, count, receiver) in [
        ("inline-member-precision-cache-small", 1, "client"),
        ("inline-member-precision-cache", 64, "client"),
        ("default-alias-cache-small", 1, "alias"),
        ("default-alias-cache", 64, "alias"),
    ] {
        let (root, graph) = graph(fixture);
        let sites = graph.call_sites_in_file(&root.join("src/mixed.mts"));
        for (member, expected) in [("get", Some("primary.get")), ("set", None)] {
            let callee = format!("{receiver}.{member}");
            let reads = sites
                .iter()
                .filter(|site| site.source_callee == callee)
                .collect::<Vec<_>>();
            assert_eq!(reads.len(), count);
            for site in reads {
                assert_eq!(
                    match &site.target {
                        ResolvedCallTarget::ModuleExport { export_path, .. } =>
                            Some(export_path.as_str()),
                        _ => None,
                    },
                    expected,
                    "{callee}: {site:?}"
                );
            }
        }
        #[cfg(feature = "test-instrumentation")]
        {
            // Baseline already constructs two generations while the anonymous
            // callee becomes known. More member reads must not add computations.
            assert_eq!(benchmark_probe_parameter_agreements(), 2);
            assert!(
                benchmark_probe_origin_nodes() <= 128 * 8 + 8,
                "member uses must share their agreement/origin DAG: {} nodes",
                benchmark_probe_origin_nodes()
            );
        }
    }
}
