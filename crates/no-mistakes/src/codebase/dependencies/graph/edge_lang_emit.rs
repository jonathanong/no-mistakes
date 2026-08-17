fn emit_lang_edges(
    facts: &LangFactMap,
    import_kind: EdgeKind,
    ref_kind: EdgeKind,
    edges: &mut Vec<Edge>,
) {
    for file in facts.files.values() {
        for import in &file.imports {
            if let Some(targets) = facts.files_by_module.get(import) {
                let scoped: std::collections::BTreeSet<_> = targets
                    .iter()
                    .filter(|target| {
                        import_kind == EdgeKind::GoImport
                            || facts
                                .files
                                .get(*target)
                                .is_some_and(|other| same_lang_package(file, other))
                    })
                    .cloned()
                    .collect();
                push_file_edges(edges, &file.path, &scoped, import_kind);
            }
        }
        for reference in &file.references {
            if let Some(targets) = facts.declarations.get(reference) {
                let scoped: std::collections::BTreeSet<_> = targets
                    .iter()
                    .filter(|target| {
                        facts
                            .files
                            .get(*target)
                            .is_some_and(|other| reference_target_allowed(file, other, reference))
                    })
                    .cloned()
                    .collect();
                push_file_edges(edges, &file.path, &scoped, ref_kind);
            }
        }
    }
}

fn emit_mod_edges(facts: &LangFactMap, kind: EdgeKind, edges: &mut Vec<Edge>) {
    for file in facts.files.values() {
        for name in &file.mods {
            let qualified = match file.module.as_deref() {
                Some(parent) => format!("{parent}.{name}"),
                None => name.clone(),
            };
            let targets = facts
                .files_by_module
                .get(&qualified)
                .or_else(|| facts.files_by_module.get(name));
            if let Some(targets) = targets {
                let scoped: std::collections::BTreeSet<_> = targets
                    .iter()
                    .filter(|target| {
                        facts
                            .files
                            .get(*target)
                            .is_some_and(|other| same_lang_package(file, other))
                    })
                    .cloned()
                    .collect();
                push_file_edges(edges, &file.path, &scoped, kind);
            }
        }
    }
}

fn emit_package_edges(facts: &LangFactMap, kind: EdgeKind, edges: &mut Vec<Edge>) {
    for files in facts.files_by_package.values() {
        let Some(root) = package_root_file(files) else {
            continue;
        };
        push_file_edges(edges, root, files, kind);
    }
}

fn package_root_file(files: &std::collections::BTreeSet<PathBuf>) -> Option<&Path> {
    let named = |want: &str| {
        files
            .iter()
            .find(|path| path.file_name().and_then(|name| name.to_str()) == Some(want))
    };
    named("lib.rs")
        .or_else(|| named("main.rs"))
        .or_else(|| named("composer.json"))
        .or_else(|| named("mod.rs"))
        .or_else(|| files.iter().next())
        .map(PathBuf::as_path)
}

fn push_file_edges(
    edges: &mut Vec<Edge>,
    source: &Path,
    targets: &std::collections::BTreeSet<PathBuf>,
    kind: EdgeKind,
) {
    for target in targets {
        if target != source {
            edges.push((NodeId::file(source), NodeId::file(target), kind));
        }
    }
}
