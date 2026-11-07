use super::*;
use crate::codebase::check_facts::{CheckFactMap, CheckFileFacts};
use crate::codebase::ts_resolver::{load_tsconfig, normalize_path};
use crate::codebase::ts_source::facts::collect_ts_facts;
use std::sync::Arc;

fn prepared() -> (PathBuf, Vec<PathBuf>, CheckFactMap) {
    let root = normalize_path(&fixture("value-import-selectors"));
    let files: Vec<_> = [
        "source.ts",
        "other.ts",
        "barrel.ts",
        "alias.ts",
        "group.ts",
        "ambiguous.ts",
        "unknown-star.ts",
        "consumer.ts",
        "unsafe.ts",
        "opaque.ts",
        "quoted.ts",
        "quoted-consumer.ts",
        "namespace-exports.ts",
        "namespace-star.ts",
        "namespace-ambiguous.ts",
        "namespace-unknown.ts",
        "namespace-unrelated.ts",
        "namespace-consumer.ts",
        "namespace-other.ts",
        "namespace-collision.ts",
        "namespace-quoted.ts",
        "namespace-quoted-star.ts",
        "external-member.ts",
        "external-member-consumer.ts",
    ]
    .iter()
    .map(|file| root.join(file))
    .collect();
    let plan = GraphBuildPlan {
        calls: true,
        imports: true,
        value_import_selectors: true,
        ..Default::default()
    };
    let fact_plan = plan.ts_fact_plan();
    assert!(!fact_plan.symbols && !fact_plan.module_bindings && !fact_plan.source);
    let ts = collect_ts_facts(&files, fact_plan)
        .into_iter()
        .map(|(file, ts)| {
            assert!(ts.parse_error.is_none(), "{file:?}: {:?}", ts.parse_error);
            assert!(ts.symbols.is_none() && ts.module_bindings.is_none());
            (
                file,
                CheckFileFacts {
                    ts: Arc::new(ts),
                    ..Default::default()
                }
                .into(),
            )
        })
        .collect();
    let facts = CheckFactMap {
        files: files.clone(),
        graph_plan: fact_plan,
        ts,
        ..Default::default()
    };
    (root, files, facts)
}

fn build(root: &Path, files: Vec<PathBuf>, facts: &CheckFactMap, selectors: bool) -> DepGraph {
    DepGraph::build_with_plan_file_list_and_check_facts(
        root,
        &load_tsconfig(&root.join("tsconfig.json")).unwrap(),
        GraphBuildPlan {
            calls: true,
            imports: true,
            value_import_selectors: selectors,
            ..Default::default()
        },
        files,
        facts,
    )
    .unwrap()
}

#[test]
fn value_import_selectors_follow_exact_specifiers_aliases_stars_and_namespaces() {
    let (root, files, facts) = prepared();
    let graph = build(&root, files, &facts, true);
    let consumer = root.join("consumer.ts");
    for (selector, producer, binding) in [
        ("selected", "source.ts", "TABLES"),
        ("open", "other.ts", "TABLES"),
        ("statement", "source.ts", "statement"),
        ("alias", "source.ts", "TABLES"),
        ("namespace.TABLES", "source.ts", "TABLES"),
        ("namespace.statement", "source.ts", "statement"),
        ("grouped.TABLES", "source.ts", "TABLES"),
        ("nested.catalog.TABLES", "source.ts", "TABLES"),
        ("fallback", "source.ts", "options"),
    ] {
        assert_eq!(
            graph.imported_value(&consumer, selector),
            Some((root.join(producer), binding.into())),
            "{selector}"
        );
    }
    for selector in [
        "ambiguous",
        "unknown",
        "unused",
        "Only",
        "run",
        "otherHelper",
        "mutable",
        "shadowed",
        "namespace",
        "namespace.absent",
    ] {
        assert!(
            graph.imported_value(&consumer, selector).is_none(),
            "{selector}"
        );
    }
    for selector in ["statement", "namespace.statement"] {
        assert_eq!(
            graph.imported_value(&root.join("unsafe.ts"), selector),
            Some((root.join("source.ts"), "statement".into()))
        );
    }
    assert!(graph
        .imported_value(&root.join("opaque.ts"), "namespace")
        .is_none());
    assert!(graph
        .imported_value(&root.join("opaque.ts"), "namespace.literal.key")
        .is_none());
    assert_eq!(
        graph.imported_value(&root.join("quoted-consumer.ts"), "dotted"),
        Some((root.join("quoted.ts"), "literal".into()))
    );
}

#[test]
fn value_import_selectors_are_opt_in_and_preserve_existing_imports_and_calls() {
    let (root, files, facts) = prepared();
    let plain = build(&root, files.clone(), &facts, false);
    let selected = build(&root, files.clone(), &facts, true);
    assert!(!GraphBuildPlan::all().value_import_selectors);
    assert!(!GraphBuildPlan::from_allowed(None).value_import_selectors);
    assert!(!GraphBuildPlan::from_allowed(Some(&[EdgeKind::Call].into())).value_import_selectors);
    let mut merged = GraphBuildPlan::default();
    merged.include(GraphBuildPlan {
        value_import_selectors: true,
        ..Default::default()
    });
    assert!(merged.value_import_selectors);
    for file in &files {
        let imports = |graph: &DepGraph| {
            graph
                .dependencies_of_node(&NodeId::file(file))
                .into_iter()
                .flatten()
                .filter(|(_, kind)| *kind != EdgeKind::SymbolReexport)
                .cloned()
                .collect::<Vec<_>>()
        };
        assert_eq!(imports(&plain), imports(&selected));
        assert_eq!(
            plain.call_sites_in_file(file),
            selected.call_sites_in_file(file)
        );
    }
    assert!(plain
        .imported_value(&root.join("consumer.ts"), "selected")
        .is_none());
    assert!(selected
        .imported_value(&root.join("consumer.ts"), "selected")
        .is_some());
}

#[path = "value_import_selectors_controls.rs"]
mod controls;

#[path = "value_import_namespaces.rs"]
mod namespaces;
