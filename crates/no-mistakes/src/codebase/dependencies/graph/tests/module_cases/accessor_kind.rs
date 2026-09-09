use super::*;
use std::collections::HashSet;

#[test]
fn paired_static_accessor_reads_keep_only_the_getter_import() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("graph-call-narrowing"));
    let tsconfig = TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    let graph =
        DepGraph::build_with_plan(&root, &tsconfig, GraphBuildPlan::imports_and_workspace())
            .unwrap();
    let deps = graph.deps_of(
        &[NodeId::file(root.join("src/paired-static-accessor-read.mts"))],
        None,
        Some(&[EdgeKind::DynamicImport].into()),
    );
    let paths: HashSet<_> = deps
        .iter()
        .filter_map(|entry| entry.node.as_file())
        .collect();

    assert!(paths.contains(
        root.join("src/paired-static-accessor-getter-loaded.mts")
            .as_path()
    ));
    assert!(!paths.contains(
        root.join("src/paired-static-accessor-setter-loaded.mts")
            .as_path()
    ));
}

#[test]
fn paired_static_accessor_writes_keep_only_the_setter_import() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("graph-call-narrowing"));
    let tsconfig = TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    let graph =
        DepGraph::build_with_plan(&root, &tsconfig, GraphBuildPlan::imports_and_workspace())
            .unwrap();
    let deps = graph.deps_of(
        &[NodeId::file(root.join("src/paired-static-accessor-write.mts"))],
        None,
        Some(&[EdgeKind::DynamicImport].into()),
    );
    let paths: HashSet<_> = deps
        .iter()
        .filter_map(|entry| entry.node.as_file())
        .collect();

    assert!(paths.contains(
        root.join("src/paired-static-accessor-setter-loaded.mts")
            .as_path()
    ));
    assert!(!paths.contains(
        root.join("src/paired-static-accessor-getter-loaded.mts")
            .as_path()
    ));
}

#[test]
fn paired_static_accessors_keep_distinct_callable_ids() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("graph-call-narrowing"));
    let tsconfig = TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    let graph = DepGraph::build_with_plan(
        &root,
        &tsconfig,
        GraphBuildPlan {
            calls: true,
            ..GraphBuildPlan::default()
        },
    )
    .unwrap();
    let source = root.join("src/paired-static-accessors.mts");
    let ids = graph
        .deps_of(
            &[NodeId::file(source.clone())],
            None,
            Some(&[EdgeKind::Call].into()),
        )
        .iter()
        .filter_map(|entry| match &entry.node {
            NodeId::Symbol {
                file,
                symbol,
                callable_id: Some(id),
                ..
            } if file.as_ref() == source.as_path() && symbol.as_ref() == "Registry/value" => {
                Some(*id)
            }
            _ => None,
        })
        .collect::<HashSet<_>>();

    assert_eq!(ids.len(), 2, "getter and setter must keep distinct identities");
}
