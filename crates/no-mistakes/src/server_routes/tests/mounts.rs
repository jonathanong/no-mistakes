#[test]
fn mount_resolver_ignores_unresolvable_and_non_relative_imports() {
    let parent = PathBuf::from("/repo/parent.ts");
    let mut parent_facts = FileFacts::default();
    parent_facts.mounts.push(MountSite {
        parent: "api".to_string(),
        child: "externalRouter".to_string(),
        prefix: "/external".to_string(),
    });
    parent_facts.imports.push(ImportBinding {
        local: "externalRouter".to_string(),
        imported: "default".to_string(),
        source: "pkg".to_string(),
    });

    let facts = HashMap::from([(parent, parent_facts)]);
    assert!(super::mounts::test_support::resolve_mounts(&facts).is_empty());
}

#[test]
fn mount_resolver_covers_import_binding_fallbacks_and_cycles() {
    let parent = PathBuf::from("/repo/parent.ts");
    let local = PathBuf::from("/repo/local.ts");
    let ambiguous = PathBuf::from("/repo/ambiguous.ts");

    let mut parent_facts = FileFacts::default();
    parent_facts.bindings.insert(
        "api".to_string(),
        Binding {
            framework: Framework::Express,
            prefixes: vec![],
        },
    );
    for child in [
        "sameName",
        "localExport",
        "localBinding",
        "aliasToExport",
        "aliasToBinding",
    ] {
        parent_facts.mounts.push(MountSite {
            parent: "api".to_string(),
            child: child.to_string(),
            prefix: format!("/{child}"),
        });
        parent_facts.imports.push(ImportBinding {
            local: child.to_string(),
            imported: if child.starts_with("alias") {
                "missing".to_string()
            } else {
                child.to_string()
            },
            source: "./local.ts".to_string(),
        });
    }
    parent_facts.mounts.push(MountSite {
        parent: "api".to_string(),
        child: "ambiguous".to_string(),
        prefix: "/ambiguous".to_string(),
    });
    parent_facts.imports.push(ImportBinding {
        local: "ambiguous".to_string(),
        imported: "missing".to_string(),
        source: "./ambiguous.ts".to_string(),
    });

    let mut local_facts = FileFacts::default();
    local_facts.bindings.insert(
        "sameName".to_string(),
        Binding {
            framework: Framework::Express,
            prefixes: vec![],
        },
    );
    local_facts.bindings.insert(
        "localBinding".to_string(),
        Binding {
            framework: Framework::Express,
            prefixes: vec![],
        },
    );
    local_facts.bindings.insert(
        "aliasToBinding".to_string(),
        Binding {
            framework: Framework::Express,
            prefixes: vec![],
        },
    );
    local_facts
        .exports
        .insert("localExport".to_string(), "localBinding".to_string());
    local_facts
        .exports
        .insert("aliasToExport".to_string(), "localBinding".to_string());

    let mut ambiguous_facts = FileFacts::default();
    ambiguous_facts
        .exports
        .insert("one".to_string(), "one".to_string());
    ambiguous_facts
        .exports
        .insert("two".to_string(), "two".to_string());

    let facts = HashMap::from([
        (parent, parent_facts),
        (local.clone(), local_facts),
        (ambiguous, ambiguous_facts),
    ]);
    let mounts = super::mounts::test_support::resolve_mounts(&facts);
    assert!(mounts
        .iter()
        .any(|mount| mount.child_file == local && mount.child == "sameName"));
    assert!(mounts
        .iter()
        .any(|mount| mount.child_file == local && mount.child == "localBinding"));

    let site = RouteSite {
        file: local.clone(),
        line: 1,
        binding: "localBinding".to_string(),
        method: "get".to_string(),
        raw_path: "/leaf".to_string(),
        path: "/leaf".to_string(),
        query_params: Vec::new(),
        framework: Framework::Express,
    };
    let prefixes = super::mounts::prefixes_for(&site, &facts, &mounts);
    assert!(prefixes.iter().any(|prefix| prefix.contains("localExport")));

    let missing_parent = [super::mounts::ResolvedMount {
        parent_file: PathBuf::from("/repo/missing.ts"),
        parent: "missing".to_string(),
        child_file: local.clone(),
        child: "localBinding".to_string(),
        prefix: "/orphan".to_string(),
    }];
    let prefixes = super::mounts::prefixes_for(&site, &facts, &missing_parent);
    assert_eq!(prefixes, vec!["/orphan"]);
}

#[test]
fn report_builder_includes_diagnostics_and_dynamic_summary() {
    let root = PathBuf::from("/repo");
    let file = root.join("api.ts");
    let mut facts = FileFacts::default();
    facts.diagnostics.push((3, "unsupported route".to_string()));
    facts.bindings.insert(
        "api".to_string(),
        Binding {
            framework: Framework::Express,
            prefixes: vec![],
        },
    );
    facts.routes.push(RouteSite {
        file: file.clone(),
        line: 4,
        binding: "api".to_string(),
        method: "get".to_string(),
        raw_path: "/users/:id".to_string(),
        path: "/users/:id".to_string(),
        query_params: Vec::new(),
        framework: Framework::Express,
    });

    let report = graph::build_report(
        &root,
        &HashMap::from([(file, facts)]),
        &crate::codebase::ts_resolver::TsConfig {
            dir: root.clone(),
            paths_dir: root.clone(),
            paths: Vec::new(),
            base_url: None,
        },
    );
    assert_eq!(report.diagnostics[0].file, "api.ts");
    assert_eq!(report.diagnostics[0].line, 3);
    assert_eq!(report.summary.dynamic_routes, 1);
}

#[test]
fn mount_resolver_covers_single_export_none_and_cycle_guards() {
    let parent = PathBuf::from("/repo/parent.ts");
    let child = PathBuf::from("/repo/child.ts");

    let mut parent_facts = FileFacts::default();
    parent_facts.bindings.insert(
        "api".to_string(),
        Binding {
            framework: Framework::Express,
            prefixes: vec!["/root".to_string()],
        },
    );
    for local in ["onlyDefault", "ambiguous"] {
        parent_facts.imports.push(ImportBinding {
            local: local.to_string(),
            imported: "default".to_string(),
            source: "./child.ts".to_string(),
        });
        parent_facts.mounts.push(MountSite {
            parent: "api".to_string(),
            child: local.to_string(),
            prefix: format!("/{local}"),
        });
    }

    let mut child_facts = FileFacts::default();
    child_facts.bindings.insert(
        "actual".to_string(),
        Binding {
            framework: Framework::Express,
            prefixes: vec![],
        },
    );
    child_facts
        .exports
        .insert("only".to_string(), "actual".to_string());

    let facts = HashMap::from([(parent.clone(), parent_facts), (child.clone(), child_facts)]);
    let mounts = super::mounts::test_support::resolve_mounts(&facts);
    assert!(mounts.iter().any(|mount| mount.child == "actual"));

    let cycle = [
        super::mounts::ResolvedMount {
            parent_file: parent.clone(),
            parent: "api".to_string(),
            child_file: child.clone(),
            child: "actual".to_string(),
            prefix: String::new(),
        },
        super::mounts::ResolvedMount {
            parent_file: child.clone(),
            parent: "actual".to_string(),
            child_file: parent.clone(),
            child: "api".to_string(),
            prefix: String::new(),
        },
    ];
    let site = RouteSite {
        file: child,
        line: 1,
        binding: "actual".to_string(),
        method: "get".to_string(),
        raw_path: "/leaf".to_string(),
        path: "/leaf".to_string(),
        query_params: Vec::new(),
        framework: Framework::Express,
    };
    let prefixes = super::mounts::prefixes_for(&site, &facts, &cycle);
    assert!(prefixes.contains(&"/".to_string()));
    assert!(prefixes.contains(&"/root".to_string()));
}
