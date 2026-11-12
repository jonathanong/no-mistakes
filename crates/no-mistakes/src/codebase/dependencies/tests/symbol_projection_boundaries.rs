use super::*;

fn saved_graph() -> (PathBuf, DepGraph, graph::SymbolIndex) {
    let root = fixture_root("simple");
    let tsconfig = TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    let graph = build_graph(&root, &tsconfig);
    let symbols = graph::SymbolIndex::build_from_root(&root, &tsconfig).unwrap();
    (root, graph, symbols)
}

#[test]
fn plain_symbol_projection_roots_respect_excluded_intermediate_files() {
    let (root, graph, symbols) = saved_graph();
    let file = root.join("c.mts");
    let roots = [Entrypoint {
        file: file.clone(),
        node: NodeId::file(file),
        symbol: None,
    }];
    let names = |entries: Vec<graph::NodeEntry>| {
        entries
            .into_iter()
            .map(|entry| entry.node.display_name(&root))
            .collect::<Vec<_>>()
    };
    let complete = resolve_symbol_dependents(&root, &roots, None, None, &graph, &symbols, None);
    assert_eq!(names(complete), ["b.mts", "a.mts"]);
    let excluded = std::collections::HashSet::from([root.join("b.mts")]);
    let pruned =
        resolve_symbol_dependents(&root, &roots, None, None, &graph, &symbols, Some(&excluded));
    assert!(pruned.is_empty(), "{pruned:?}");
    let shallow = resolve_symbol_dependents(
        &root,
        &roots,
        Some(1),
        None,
        &graph,
        &symbols,
        Some(&std::collections::HashSet::new()),
    );
    assert_eq!(names(shallow), ["b.mts"]);
}

#[test]
fn dependency_json_and_yaml_serializers_preserve_the_same_graph_records() {
    let (root, graph, symbols) = saved_graph();
    let file = root.join("c.mts");
    let roots = [Entrypoint {
        file: file.clone(),
        node: NodeId::file(file),
        symbol: None,
    }];
    let entries = resolve_symbol_dependents(&root, &roots, None, None, &graph, &symbols, None);
    let labels = vec!["c.mts".to_owned()];
    let mut json = Vec::new();
    write_entries(Format::Json, &labels, &entries, &root, &mut json).unwrap();
    let mut yaml = Vec::new();
    write_entries(Format::Yml, &labels, &entries, &root, &mut yaml).unwrap();
    let json: serde_json::Value = serde_json::from_slice(&json).unwrap();
    let yaml: serde_yaml::Value = serde_yaml::from_slice(&yaml).unwrap();
    assert_eq!(json, serde_json::to_value(yaml).unwrap());
    assert_eq!(json["roots"], serde_json::json!(["c.mts"]));
    let files = json["files"].as_array().unwrap();
    assert_eq!(files.len(), 2);
    assert_eq!(files[0]["path"], "b.mts");
    assert_eq!(files[0]["depth"], 1);
    assert_eq!(files[1]["path"], "a.mts");
    assert_eq!(files[1]["depth"], 2);
}
