use super::config::sink;
use super::traces::{node_label, shortest};
use super::*;
use crate::config::v2::schema::EffectKindConfig;
fn root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/query-reached-per-item/fixture")
}
#[test]
fn iteration_helpers_share_the_canonical_graph_and_report_shortest_paths() {
    let findings = crate::codebase::rules::run_check(&root(), None, None).unwrap();
    let findings = findings
        .into_iter()
        .filter(|finding| finding.rule == RULE_ID)
        .collect::<Vec<_>>();
    for line in [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 19, 21, 22] {
        assert!(
            findings
                .iter()
                .any(|finding| finding.file == "src/entry.mts" && finding.line == line),
            "missing line {line}: {findings:#?}"
        );
    }
    assert!(
        findings
            .iter()
            .any(|finding| finding.line == 5 && finding.message.contains("lookup → inner → read")),
        "{findings:#?}"
    );
    assert!(
        findings
            .iter()
            .any(|finding| finding.line == 11 && finding.message.contains("forEach callback")),
        "{findings:#?}"
    );
    assert!(
        findings.iter().any(
            |finding| finding.line == 21 && finding.message.contains("transaction client only")
        ),
        "{findings:#?}"
    );
    for line in [23, 26, 28, 29, 42, 44] {
        assert!(
            !findings
                .iter()
                .any(|finding| finding.line == line && finding.file.ends_with("entry.mts")),
            "{findings:#?}"
        );
    }
    assert_eq!(
        findings,
        crate::codebase::rules::run_check(&root(), None, None)
            .unwrap()
            .into_iter()
            .filter(|finding| finding.rule == RULE_ID)
            .collect::<Vec<_>>()
    );
}
#[test]
fn configured_baseline_and_effect_categories_preserve_occurrences() {
    let root = root();
    let run =
        |name| crate::codebase::rules::run_check(&root, Some(&root.join(name)), None).unwrap();
    let allowed = run("allow.yml");
    assert!(!allowed
        .iter()
        .any(|finding| finding.file == "src/entry.mts" && finding.line == 5));
    assert!(!allowed.iter().any(
        |finding| finding.file == "src/entry.mts" && finding.import.as_deref() == Some("read")
    ));
    assert!(
        allowed
            .iter()
            .any(|finding| finding.file == "src/entry.mts" && finding.line == 7),
        "{allowed:#?}"
    );
    assert_eq!(run("category.yml"), run(".no-mistakes.yml"));
    assert!(run("filtered.yml")
        .iter()
        .all(|finding| finding.file == "src/entry.mts"));
    assert!(run("disabled.yml").is_empty());
    let unknown = crate::codebase::rules::run_check(&root, Some(&root.join("unknown.yml")), None)
        .unwrap_err();
    assert!(unknown
        .to_string()
        .contains("unknown effects kind `absent`"));
    for name in [
        "missing.yml",
        "unknown.yml",
        "bad-file.yml",
        "bad-line.yml",
        "null-options.yml",
    ] {
        assert!(
            crate::codebase::rules::run_check(&root, Some(&root.join(name)), None).is_err(),
            "{name}"
        );
    }
}
#[test]
fn shortest_path_and_spelling_helpers_are_deterministic() {
    let effect = EffectKindConfig {
        functions: vec!["read".into()],
        transaction_functions: vec!["tx.query".into()],
        batch_functions: vec!["batch".into()],
        ..Default::default()
    };
    assert_eq!(sink(&effect, "client.read"), Some(false));
    assert_eq!(sink(&effect, "tx.query"), Some(true));
    assert_eq!(sink(&effect, "other.query"), None);
    assert!(exempt(&effect, "client.batch"));
    assert!(!exempt(&effect, "lookup"));
    assert!(shortest(std::iter::empty()).is_none());
    assert_eq!(
        shortest(
            [
                Reach {
                    path: vec!["z".into()],
                    transaction: true
                },
                Reach {
                    path: vec!["a".into(), "b".into()],
                    transaction: false
                }
            ]
            .into_iter()
        )
        .unwrap()
        .path,
        ["z"]
    );
}

#[test]
fn transaction_only_and_alternate_batch_paths_have_distinct_diagnostics() {
    let findings = crate::codebase::rules::run_check(&root(), None, None).unwrap();
    let lookup = |name| {
        findings
            .iter()
            .find(|finding| {
                finding.import.as_deref() == Some(name) && finding.file == "src/entry.mts"
            })
            .unwrap()
    };
    assert!(lookup("transactional")
        .message
        .contains("transaction client only"));
    assert!(!lookup("mixed").message.contains("transaction client only"));
    assert!(lookup("batchAndRead")
        .message
        .contains("batchAndRead → read"));
    assert!(!findings
        .iter()
        .any(|finding| matches!(finding.import.as_deref(), Some("batchAlias" | "onlyBatch"))));
    assert!(!findings
        .iter()
        .any(|finding| finding.file == "src/disabled.mts"));
    assert_eq!(node_label(&NodeId::file("module.mts")), "module");
}
#[test]
fn per_item_metadata_is_explicit_demand_and_display_roots_remain_supported() {
    let file = root().join("src/entry.mts");
    let facts = crate::codebase::ts_source::facts::collect_ts_facts(
        std::slice::from_ref(&file),
        crate::codebase::ts_source::facts::TsFactPlan::imports(),
    );
    let facts = facts.get(&file).unwrap();
    assert!(!facts.function_calls.is_empty());
    assert!(facts.per_item_calls.is_empty());
    let site = ResolvedCallSite {
        file,
        caller: Some("display".into()),
        caller_id: None,
        line: 1,
        offset: 1,
        invocation: crate::codebase::dependencies::extract::InvocationKind::Call,
        source_callee: "callee".into(),
        target: crate::codebase::dependencies::graph::ResolvedCallTarget::Unknown,
        alternatives: Vec::new(),
        returned_from: Vec::new(),
        typed_from: Vec::new(),
        returned_member_from: Vec::new(),
        target_node: None,
    };
    assert_eq!(source_node(&site), NodeId::symbol(&site.file, "display"));
}
#[test]
fn batch_wrappers_do_not_exempt_eager_queries_or_shadowed_aliases() {
    let findings = crate::codebase::rules::run_check(&root(), None, None).unwrap();
    assert!(findings
        .iter()
        .any(|finding| finding.file == "src/entry.mts" && finding.line == 60));
    assert!(findings
        .iter()
        .any(|finding| finding.file == "src/entry.mts" && finding.line == 63));
    // The loop header is evaluated before the per-item body; only the helper
    // invocation in the body should be reported on this source line.
    assert!(!findings
        .iter()
        .any(|finding| finding.file == "src/entry.mts"
            && finding.line == 68
            && finding.import.as_deref() == Some("read")));
}
#[test]
fn repeated_calls_and_named_callbacks_on_one_line_remain_distinct() {
    let findings = crate::codebase::rules::run_check(&root(), None, None).unwrap();
    for line in [70, 71] {
        let occurrences = findings
            .iter()
            .filter(|finding| finding.file == "src/entry.mts" && finding.line == line)
            .collect::<Vec<_>>();
        assert_eq!(occurrences.len(), 2, "{occurrences:#?}");
        assert!(occurrences[0].message.ends_with("(occurrence 1)"));
        assert!(occurrences[1].message.ends_with("(occurrence 2)"));
    }
}
#[test]
fn statically_named_configured_member_callbacks_are_effect_occurrences() {
    let findings = crate::codebase::rules::run_check(&root(), None, None).unwrap();
    assert!(findings
        .iter()
        .any(|finding| finding.file == "src/entry.mts"
            && finding.line == 72
            && finding.import.as_deref() == Some("client.read")));
}
#[test]
fn named_const_alias_callbacks_keep_their_actual_binding_use_offset() {
    let findings = crate::codebase::rules::run_check(&root(), None, None).unwrap();
    for line in [74, 75] {
        assert!(
            findings
                .iter()
                .any(|finding| finding.file == "src/entry.mts"
                    && finding.line == line
                    && finding.import.as_deref() == Some("lookupAlias")),
            "{findings:#?}"
        );
    }
}

#[test]
fn binding_targets_match_imports_callbacks_and_transitive_sinks_without_name_collisions() {
    let root = root().with_file_name("targets-fixture");
    let findings = crate::codebase::rules::run_check(&root, None, None).unwrap();
    assert_eq!(
        findings
            .iter()
            .map(|finding| (finding.file.as_str(), finding.line))
            .collect::<Vec<_>>(),
        [11, 12, 13, 14, 16, 18, 19, 20, 21, 23, 24, 25, 26, 27, 28, 29]
            .map(|line| ("src/entry.mts", line))
    );
    assert!(findings
        .iter()
        .any(|finding| finding.line == 28 && finding.message.contains("lookup → inner → load")));
    assert!(findings.iter().any(
        |finding| finding.line == 29 && finding.message.contains("cachedLookup → valkey.hget")
    ));
    assert!(findings.iter().any(|finding| finding.line == 24
        && finding.message.contains("forEach callback")
        && finding
            .target
            .as_deref()
            .is_some_and(|target| target.starts_with("valkey:"))));
}

#[test]
fn binding_targets_require_nonempty_module_and_export_selectors() {
    let root = root().with_file_name("targets-fixture");
    for name in ["invalid-module.yml", "invalid-export.yml"] {
        let error =
            crate::codebase::rules::run_check(&root, Some(&root.join(name)), None).unwrap_err();
        assert!(error.to_string().contains("non-empty module and export"));
    }
}

#[test]
fn legacy_transaction_spellings_take_precedence_over_binding_targets() {
    let root = root().with_file_name("targets-fixture");
    let findings =
        crate::codebase::rules::run_check(&root, Some(&root.join("transaction.yml")), None)
            .unwrap();
    assert_eq!(findings.len(), 2, "{findings:#?}");
    assert!(findings
        .iter()
        .any(|finding| finding.line == 13 && finding.message.contains("transaction client only")));
    assert!(findings
        .iter()
        .any(|finding| finding.line == 18 && !finding.message.contains("transaction client only")));
}

#[test]
fn adding_binding_targets_retains_legacy_lists_without_duplicate_occurrences() {
    let root = root().with_file_name("targets-fixture");
    let findings =
        crate::codebase::rules::run_check(&root, Some(&root.join("mixed.yml")), None).unwrap();
    // Legacy spellings intentionally still select the shadowed parameter at 38.
    assert_eq!(
        findings
            .iter()
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        [11, 14, 16, 19, 20, 26, 28, 38]
    );
}

#[test]
fn callback_occurrences_keep_their_own_bindings_within_one_caller() {
    let root = root().with_file_name("callbacks-fixture");
    let findings = crate::codebase::rules::run_check(&root, None, None).unwrap();
    assert!(findings
        .iter()
        .all(|finding| finding.file == "src/entry.mts"));
    assert_eq!(
        findings
            .iter()
            .map(|finding| (finding.line, finding.import.as_deref().unwrap()))
            .collect::<Vec<_>>(),
        [
            (6, "read"),
            (13, "cache.get"),
            (20, "lookup"),
            (27, "read"),
            (27, "read"),
            (29, "read"),
        ]
    );
    let repeated = findings
        .iter()
        .filter(|finding| finding.line == 27)
        .collect::<Vec<_>>();
    assert!(repeated[0].message.ends_with("(occurrence 1)"));
    assert!(repeated[1].message.ends_with("(occurrence 2)"));
    assert!(findings
        .iter()
        .find(|finding| finding.line == 29)
        .unwrap()
        .message
        .contains("map callback"));
}

#[test]
fn returned_executors_factory_exports_and_client_parameters_keep_provenance() {
    let root = root().with_file_name("provenance-fixture");
    let findings = crate::codebase::rules::run_check(&root, None, None).unwrap();
    assert_eq!(
        findings
            .iter()
            .filter(|finding| finding.file == "src/entry.mts")
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        (8..=15).chain([26]).collect::<Vec<_>>(),
        "{findings:#?}"
    );
    assert!(
        findings
            .iter()
            .any(|finding| finding.file == "src/helpers.mts" && finding.line == 3),
        "{findings:#?}"
    );
}

#[test]
fn value_provenance_rejects_reassignments_conflicts_and_unrelated_receivers() {
    let root = root().with_file_name("provenance-fixture");
    let findings = crate::codebase::rules::run_check(&root, None, None).unwrap();
    assert_eq!(
        findings
            .iter()
            .filter(|finding| finding.file == "src/controls.mts")
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        [11, 17, 24, 27, 39],
        "{findings:#?}"
    );
}

#[test]
fn returned_executor_selectors_mark_visible_factories_without_selecting_creation_calls() {
    let root = root().with_file_name("provenance-fixture");
    let findings =
        crate::codebase::rules::run_check(&root, Some(&root.join("returned-only.yml")), None)
            .unwrap();
    assert_eq!(
        findings
            .iter()
            .filter(|finding| finding.file == "src/entry.mts")
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        [8, 9],
        "{findings:#?}"
    );
    assert!(findings
        .iter()
        .all(|finding| finding.message.contains("transaction client only")));
}

#[test]
fn review_regressions_preserve_call_inputs_returns_mutations_and_factory_origins() {
    let root = root().with_file_name("provenance-fixture");
    let findings =
        crate::codebase::rules::run_check(&root, Some(&root.join("review.yml")), None).unwrap();
    let review = findings
        .iter()
        .filter(|finding| finding.file == "src/review.mts")
        .collect::<Vec<_>>();
    assert_eq!(
        review
            .iter()
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        [9, 25, 26, 28],
        "{findings:#?}"
    );
    assert!(review
        .iter()
        .filter(|finding| finding.line != 9)
        .all(|finding| finding.message.contains("transaction client only")));
    let wrapper =
        crate::codebase::rules::run_check(&root, Some(&root.join("wrapper-only.yml")), None)
            .unwrap();
    assert_eq!(
        wrapper
            .iter()
            .filter(|finding| finding.file == "src/review.mts")
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        [25],
        "{wrapper:#?}"
    );
    let marker =
        crate::codebase::rules::run_check(&root, Some(&root.join("review-marker.yml")), None)
            .unwrap();
    assert!(
        marker.iter().any(|finding| finding.file == "src/review.mts"
            && finding.line == 28
            && finding.message.contains("transaction client only")),
        "{marker:#?}"
    );
}

#[test]
fn named_client_configuration_reaches_defaults_and_arguments_of_factory_clients() {
    let root = root().with_file_name("provenance-fixture");
    let findings =
        crate::codebase::rules::run_check(&root, Some(&root.join("named-clients.yml")), None)
            .unwrap();
    assert_eq!(
        findings
            .iter()
            .filter(|finding| finding.file == "src/local-client-entry.mts")
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        [4, 5],
        "{findings:#?}"
    );
    let report = crate::effects_query::run(
        &root,
        Some(&root.join("named-clients.yml")),
        None,
        "valkey",
        Path::new("src/local-client-entry.mts"),
        &[],
        None,
    )
    .unwrap();
    assert_eq!(
        report
            .call_sites
            .iter()
            .map(|site| site.callee.as_str())
            .collect::<Vec<_>>(),
        ["primary.get", "session.info", "session.scan"]
    );
}

#[test]
fn mutation_aliases_completion_adapters_constructors_and_redeclarations_keep_provenance_sound() {
    let root = root().with_file_name("provenance-fixture");
    let findings = crate::codebase::rules::run_check(&root, None, None).unwrap();
    assert_eq!(
        findings
            .iter()
            .filter(|finding| finding.file == "src/adapters-and-mutations.mts")
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        [11, 15, 19, 23, 29],
        "{findings:#?}"
    );
}

#[path = "tests/projections.rs"]
mod projections;

#[path = "tests/typed_clients.rs"]
mod typed_clients;

mod namespace_shadows;

#[path = "tests/superseded_provenance.rs"]
mod superseded_provenance;
#[path = "tests/type_import_order.rs"]
mod type_import_order;
