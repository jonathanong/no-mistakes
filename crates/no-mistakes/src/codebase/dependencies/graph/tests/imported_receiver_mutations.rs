use super::*;

#[test]
fn callee_owned_receiver_defaults_keep_their_execution_context() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture(
        "value-flow-default-receiver-context",
    ));
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
    let sites = graph.call_sites_in_file(&root.join("src/entry.mts"));
    for (caller, export) in [
        ("beforeWrite", Some("primary.get")),
        ("afterWrite", None),
        ("externalWrite", None),
        ("objectDefault", Some("scoped.get")),
    ] {
        let site = sites
            .iter()
            .find(|site| {
                site.caller.as_deref() == Some(caller) && site.source_callee == "client.get"
            })
            .unwrap();
        assert_eq!(
            match &site.target {
                ResolvedCallTarget::ModuleExport { export_path, .. } => Some(export_path.as_str()),
                _ => None,
            },
            export,
            "{caller}: {site:?}"
        );
    }
    let factory = graph.call_sites_in_file(&root.join("src/source.mts"));
    let factory = factory
        .iter()
        .find(|site| site.source_callee == "primary.info")
        .unwrap();
    let caller = graph.call_sites_in_file(&root.join("src/cross-file.mts"));
    let caller = caller
        .iter()
        .find(|site| site.source_callee == "client.get")
        .unwrap();
    // Equal file-local IDs must not turn another module's acquisition into a
    // callee-owned default read. The factory mutates before returning its object.
    assert_eq!(factory.caller_id, caller.caller_id);
    assert!(factory.caller_id.is_some());
    assert!(
        matches!(&factory.target, ResolvedCallTarget::ModuleExport { export_path, .. }
        if export_path == "primary.info")
    );
    assert!(
        matches!(caller.target, ResolvedCallTarget::Unknown),
        "{caller:?}"
    );
}

#[test]
fn imported_receivers_preserve_member_snapshots_and_live_nested_inputs() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/query-reached-per-item/provenance-fixture"),
    );
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
    let sites = graph.call_sites_in_file(&root.join("src/imported-receiver-mutations.mts"));
    for (caller, callee, export) in [
        (Some("beforeWrite"), "client.get", "primary.get"),
        (Some("untouchedMethod"), "client.info", "primary.info"),
        (None, "copiedInfoAfter", "primary.info"),
        (None, "primary.info", "primary.info"),
        (None, "nestedCopied", "primary.nested.get"),
        (
            Some("untouchedNested"),
            "client.other.get",
            "primary.other.get",
        ),
    ] {
        let site = sites
            .iter()
            .find(|site| site.caller.as_deref() == caller && site.source_callee == callee)
            .unwrap();
        assert!(
            matches!(&site.target, ResolvedCallTarget::ModuleExport { export_path, .. }
            if export_path == export),
            "{caller:?} {callee}: {site:?}"
        );
    }
    for (caller, callee) in [
        ("lateDefault", "client.get"),
        ("earlyObject", "client.get"),
        ("suppliedObject", "client.get"),
        ("suppliedIdentifier", "client.get"),
        ("lateIdentifier", "client.get"),
        ("copiedAfter", "get"),
        ("identifierObject", "options.client.get"),
        ("nestedSupplied", "client.get"),
        ("captured", "client.get"),
        ("identifierBefore", "options.client.get"),
        ("nestedIdentifier", "options.box.client.get"),
        ("nestedReceiver", "client.get"),
        ("dynamicReceiver", "client.bucket.get"),
        ("mergedReceiver", "client.get"),
    ] {
        let site = sites
            .iter()
            .find(|site| site.caller.as_deref() == Some(caller) && site.source_callee == callee)
            .unwrap();
        assert!(
            matches!(site.target, ResolvedCallTarget::Unknown),
            "{caller} {callee}: {site:?}"
        );
    }
}

#[test]
fn nested_alias_writes_invalidate_original_receivers_with_member_precision() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/query-reached-per-item/provenance-fixture"),
    );
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
    let sites = graph.call_sites_in_file(&root.join("src/nested-receiver-writes.mts"));
    for (caller, callee, export) in [
        (Some("before"), "client.get", Some("primary.get")),
        (Some("scan"), "client.get", None),
        (Some("untouched"), "client.info", Some("primary.info")),
        (Some("other"), "client.info", Some("session.info")),
        (Some("snapshot"), "get", Some("primary.get")),
        (Some("objectSnapshot"), "options.get", Some("primary.get")),
        (Some("copiedAfter"), "get", None),
        (Some("nestedWrite"), "client.nested.get", None),
        (
            Some("sibling"),
            "client.other.get",
            Some("primary.other.get"),
        ),
        (None, "nestedCopied", Some("primary.nested.get")),
        (Some("detachedReceiver"), "client.get", Some("primary.get")),
        (Some("dynamicWrite"), "client.get", None),
        (Some("mutableReceiver"), "client.get", Some("primary.get")),
        (Some("savedReceiver"), "client.get", None),
        (Some("liveReceiver"), "client.get", Some("primary.get")),
        (Some("reboundAlias"), "client.get", Some("primary.get")),
        (Some("calledReceiver"), "client.get", None),
        (Some("mixedReceiver"), "client.get", None),
        (Some("inverseReceiver"), "client.get", None),
    ] {
        let site = sites
            .iter()
            .find(|site| site.caller.as_deref() == caller && site.source_callee == callee)
            .unwrap();
        assert_eq!(
            match &site.target {
                ResolvedCallTarget::ModuleExport { export_path, .. } => Some(export_path.as_str()),
                _ => None,
            },
            export,
            "{caller:?} {callee}: {site:?}"
        );
    }
}

#[cfg(feature = "test-instrumentation")]
#[test]
fn lazy_imported_receiver_sites_share_one_agreement_and_context_dag() {
    for (name, callee) in [
        ("value-flow-imported-receiver-cache", "options.client.get"),
        ("value-flow-destructured-acquisition-cache", "get"),
    ] {
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
        let sites = graph.call_sites_in_file(&root.join("src/projected.mts"));
        assert_eq!(sites.len(), 129);
        let receivers = sites
            .iter()
            .filter(|site| site.source_callee == callee)
            .collect::<Vec<_>>();
        assert_eq!(receivers.len(), 128);
        assert!(receivers
            .iter()
            .all(|site| matches!(site.target, ResolvedCallTarget::Unknown)));
        assert_eq!(benchmark_probe_parameter_agreements(), 1);
        assert!(
            benchmark_probe_origin_nodes() <= 128 * 8 + 8,
            "{name}: agreement/context must stay shared: {} nodes",
            benchmark_probe_origin_nodes()
        );
    }
}

#[test]
fn destructured_methods_acquire_receivers_at_the_callers_invocation() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture(
        "value-flow-destructured-acquisition",
    ));
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
    let sites = graph.call_sites_in_file(&root.join("src/entry.mts"));
    for (caller, callee, expected) in [
        ("beforeWrite", "get", Some("primary.get")),
        ("afterWrite", "get", None),
        ("copiedMethod", "get", Some("primary.get")),
        ("untouched", "info", Some("primary.info")),
        ("nested", "get", None),
        ("forwarded", "get", None),
        ("defaulted", "get", None),
        ("arrayInput", "get", None),
        ("ownedDefault", "get", Some("session.get")),
    ] {
        let site = sites
            .iter()
            .find(|site| site.caller.as_deref() == Some(caller) && site.source_callee == callee)
            .unwrap();
        match expected {
            Some(expected) => assert!(
                matches!(&site.target,
                ResolvedCallTarget::ModuleExport { export_path, .. }
                if export_path == expected),
                "{caller}: {site:?}"
            ),
            None => assert!(
                matches!(site.target, ResolvedCallTarget::Unknown),
                "{caller}: {site:?}"
            ),
        }
    }
}
