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
        (
            "argument-order",
            Some("before"),
            "client.get",
            Some("primary.get"),
        ),
        ("argument-order", Some("during"), "client.get", None),
        (
            "argument-order",
            Some("during"),
            "client.set",
            Some("primary.set"),
        ),
        (
            "argument-order",
            Some("otherReceiver"),
            "client.get",
            Some("replica.get"),
        ),
        (
            "argument-order",
            Some("after"),
            "client.get",
            Some("later.get"),
        ),
        (
            "argument-order",
            Some("copiedMethod"),
            "get",
            Some("copied.get"),
        ),
        (
            "argument-order",
            Some("copiedObject"),
            "options.get",
            Some("objectCopy.get"),
        ),
        ("argument-order", Some("inlineDuring"), "client.get", None),
        (
            "argument-order",
            Some("inlineCopied"),
            "get",
            Some("inlineCopy.get"),
        ),
        (
            "argument-order",
            Some("factory/returnedDuring"),
            "client.get",
            None,
        ),
        ("argument-order", Some("boundDuring"), "client.get", None),
        (
            "argument-order",
            Some("boundCopied"),
            "get",
            Some("boundCopy.get"),
        ),
        (
            "argument-order",
            Some("defaultReceiver"),
            "client.get",
            None,
        ),
        ("argument-order", Some("inlineDefault"), "get", None),
        (
            "argument-order",
            Some("assignmentRhs"),
            "client.get",
            Some("rhs.get"),
        ),
        ("argument-order", Some("afterRhs"), "client.get", None),
        ("argument-order", Some("spreadUnknown"), "client.get", None),
        (
            "nested-argument-order",
            Some("nestedDefault"),
            "fn",
            Some("primary.get"),
        ),
        (
            "nested-argument-order",
            Some("nestedArgument"),
            "fn",
            Some("supplied.get"),
        ),
        (
            "nested-argument-order",
            Some("nestedDefaultLater"),
            "fn",
            None,
        ),
        (
            "nested-argument-order",
            Some("nestedArgumentLater"),
            "fn",
            Some("suppliedLater.get"),
        ),
        (
            "nested-argument-order",
            Some("forwardedDefault"),
            "fn",
            Some("forwarded.get"),
        ),
        (
            "nested-argument-order",
            Some("objectDefault"),
            "fn",
            Some("objectSource.get"),
        ),
        (
            "nested-argument-order",
            Some("snapshotDefault"),
            "fn",
            Some("snapshot.get"),
        ),
        (
            "nested-argument-order",
            Some("declarationAfterCall"),
            "fn",
            None,
        ),
        (
            "nested-argument-order",
            Some("localDeclarationAfterReturn"),
            "fn",
            None,
        ),
        (
            "nested-argument-order",
            Some("mutableArgument"),
            "fn",
            Some("mutableBefore.get"),
        ),
        ("nested-argument-order", Some("mutableDefault"), "fn", None),
        (
            "nested-argument-order",
            Some("objectDefaultLater"),
            "fn",
            None,
        ),
        (
            "nested-copied-fields",
            Some("mutableObjectArgument"),
            "object.get",
            None,
        ),
        (
            "nested-copied-fields",
            Some("mutableObjectDefault"),
            "object.get",
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
