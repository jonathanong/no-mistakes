#[test]
fn graph_uses_standard_symbols_instead_of_legacy_list_symbols() {
    use crate::codebase::check_facts::{CheckFactMap, CheckFileFacts};
    use crate::codebase::ts_source::facts::TsFileFacts;
    use crate::codebase::ts_symbols::{Export, FileSymbols};

    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/napi/analyze-project-mixed-symbol-parse-modes"),
    );
    let file = root.join("types.js");
    assert!(file.is_file(), "fixture must remain on disk");

    let standard = FileSymbols {
        exports: vec![Export {
            name: "javascriptValue".to_string(),
            local: None,
            kind: ExportKind::Const,
            line: 7,
            is_type_only: false,
        }],
        imports: Vec::new(),
    };
    let legacy = FileSymbols {
        exports: vec![Export {
            name: "JavaScriptShape".to_string(),
            local: None,
            kind: ExportKind::Interface,
            line: 3,
            is_type_only: true,
        }],
        imports: Vec::new(),
    };
    let mut facts = CheckFactMap::default();
    facts.ts.insert(
        file.clone(),
        Arc::new(CheckFileFacts {
            ts: Arc::new(TsFileFacts {
                symbols: Some(standard.clone()),
                ..TsFileFacts::default()
            }),
            symbols: Some(Arc::new(standard)),
            legacy_symbols: Some(Arc::new(legacy)),
            ..CheckFileFacts::default()
        }),
    );

    let visible = HashSet::from([file.clone()]);
    let tsconfig = TsConfig {
        dir: root.clone(),
        paths: Vec::new(),
        paths_dir: root.clone(),
        base_url: None,
    };
    let resolver = ImportResolver::new(&tsconfig).with_visible(&visible);
    let edges = collect_symbol_edges(
        &root,
        SymbolGraphFiles {
            indexable: std::slice::from_ref(&file),
            all: std::slice::from_ref(&file),
            visible: &visible,
        },
        &facts,
        &resolver,
        &Default::default(),
        None,
    );

    let standard_node = NodeId::Symbol {
        file: file.clone(),
        symbol: "javascriptValue".to_string(),
    };
    assert!(edges.contains(&(
        NodeId::File(file.clone()),
        standard_node,
        EdgeKind::Import,
    )));
    assert!(edges.iter().any(|(_, node, _)| matches!(
        node,
        NodeId::Symbol { symbol, .. } if symbol == "javascriptValue"
    )));
    assert!(!edges.iter().any(|(from, to, _)| [from, to].iter().any(|node| matches!(
        node,
        NodeId::Symbol { symbol, .. } if symbol == "JavaScriptShape"
    ))));
}
