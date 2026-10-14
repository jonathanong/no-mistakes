use super::*;

#[test]
fn imported_receiver_reads_follow_assignment_and_invocation_order() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture(
        "value-flow-receiver-evaluation-order",
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
    for (file, caller, callee, export) in [
        (
            "supplied-context",
            Some("before"),
            "client.get",
            Some("primary.get"),
        ),
        ("supplied-context", Some("after"), "client.get", None),
        (
            "supplied-context",
            Some("objectBefore"),
            "value.client.get",
            Some("scoped.get"),
        ),
        (
            "supplied-context",
            Some("objectAfter"),
            "value.client.get",
            None,
        ),
        ("rhs", Some("rhs"), "client.get", Some("primary.get")),
        ("rhs", Some("afterWrite"), "client.get", None),
        ("rhs", Some("iterable"), "client.get", Some("replica.get")),
        ("rhs", Some("keys"), "client.get", Some("scoped.get")),
        (
            "forwarded",
            Some("inner"),
            "client.get",
            Some("primary.get"),
        ),
        ("forwarded", Some("innerAfter"), "client.get", None),
        (
            "object-forwarded",
            Some("inner"),
            "client.get",
            Some("primary.get"),
        ),
        (
            "object-forwarded",
            Some("identifier"),
            "options.client.get",
            Some("primary.get"),
        ),
        ("object-forwarded", Some("innerAfter"), "client.get", None),
        (
            "object-forwarded",
            Some("identifierAfter"),
            "options.client.get",
            None,
        ),
        // Existing default-source context summaries remain conservative across helpers.
        ("default-forwarded", Some("inner"), "client.get", None),
        ("default-forwarded", Some("innerAfter"), "client.get", None),
        ("returned", None, "client().get", Some("primary.get")),
        ("returned", None, "clientAfter().get", None),
        ("returned", None, "saved", Some("replica.get")),
        ("returned", None, "aliasSaved", Some("scoped.get")),
        ("returned", None, "localSaved", Some("local.get")),
        ("returned", None, "outsideSaved", Some("standalone.get")),
        ("returned", None, "nestedSaved", Some("nested.get")),
        ("owned-fields", None, "object.get", Some("primary.get")),
        (
            "owned-fields",
            Some("defaultConsume"),
            "options.get",
            Some("primary.get"),
        ),
        (
            "owned-live",
            Some("before"),
            "options.client.get",
            Some("primary.get"),
        ),
        ("owned-live", Some("after"), "options.client.get", None),
        (
            "owned-live",
            Some("defaultAfter"),
            "options.client.get",
            None,
        ),
        ("owned-fields", None, "objectAfter.get", None),
        ("owned-fields", None, "outside.get", Some("replica.get")),
        ("owned-fields", None, "localObject.get", Some("scoped.get")),
        ("owned-fields", None, "beforeObject.get", None),
        ("owned-fields", Some("consume"), "options.get", None),
        (
            "owned-fields",
            Some("manyConsume"),
            "options.get",
            Some("nested.get"),
        ),
        (
            "method-defaults",
            Some("before"),
            "get",
            Some("primary.get"),
        ),
        ("method-defaults", Some("after"), "get", None),
        (
            "method-defaults",
            Some("localWrite"),
            "get",
            Some("replica.get"),
        ),
        (
            "method-defaults",
            Some("suppliedUndefined"),
            "get",
            Some("scoped.get"),
        ),
        (
            "method-defaults",
            Some("fallback"),
            "get",
            Some("local.get"),
        ),
        ("method-defaults", Some("fallbackAfter"), "get", None),
        (
            "provenance-receivers",
            Some("objectBefore"),
            "value.client.get",
            Some("primary.get"),
        ),
        (
            "provenance-receivers",
            Some("arrayBefore"),
            "value.0.get",
            Some("replica.get"),
        ),
        (
            "provenance-receivers",
            Some("objectAfter"),
            "value.client.get",
            None,
        ),
        (
            "provenance-receivers",
            Some("arrayAfter"),
            "value.0.get",
            None,
        ),
        (
            "provenance-receivers",
            Some("copiedAfter"),
            "value.get",
            Some("primary.get"),
        ),
        // Local aliases are read live at the member access; copied methods remain snapshots.
        (
            "live-parameter-alias",
            Some("liveAfterWrite"),
            "chained.get",
            None,
        ),
        (
            "live-parameter-alias",
            Some("aliasBeforeWrite"),
            "alias.get",
            Some("primary.get"),
        ),
        (
            "live-parameter-alias",
            Some("copiedMethod"),
            "get",
            Some("primary.get"),
        ),
        (
            "live-parameter-alias",
            Some("unrelated"),
            "alias.set",
            Some("replica.set"),
        ),
        (
            "live-parameter-alias",
            Some("objectAlias"),
            "alias.client.get",
            None,
        ),
    ] {
        let sites = graph.call_sites_in_file(&root.join(format!("src/{file}.mts")));
        let line = match callee {
            "client().get" => Some(4),
            "clientAfter().get" => Some(8),
            _ => None,
        };
        let recorded = if line.is_some() {
            "<unknown>.get"
        } else {
            callee
        };
        let site = sites
            .iter()
            .find(|site| {
                site.caller.as_deref() == caller
                    && site.source_callee == recorded
                    && line.is_none_or(|line| site.line == line)
            })
            .unwrap_or_else(|| panic!("{file} {caller:?} {callee}: {sites:?}"));
        assert_eq!(
            match &site.target {
                ResolvedCallTarget::ModuleExport { export_path, .. } => Some(export_path.as_str()),
                _ => None,
            },
            export,
            "{file} {caller:?} {callee}: {site:?}"
        );
    }
}
