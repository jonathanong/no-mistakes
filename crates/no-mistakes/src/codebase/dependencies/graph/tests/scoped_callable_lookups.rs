#[test]
fn scoped_names_preserve_last_binding_and_distinguish_lexical_frames() {
    let names = index_scoped_names([
        ((0, "run".to_string()), 1),
        ((1, "run".to_string()), 2),
        ((0, "run".to_string()), 3),
    ]);
    let borrowed = String::from("run");
    assert_eq!(scoped_name(&names, 0, &borrowed), Some(&3));
    assert_eq!(scoped_name(&names, 1, &borrowed), Some(&2));
    assert!(scoped_name(&names, 2, &borrowed).is_none());
    assert!(scoped_name(&names, 0, "missing").is_none());
    assert_eq!(names.values().map(|names| names.len()).sum::<usize>(), 2);
    assert!(names
        .values()
        .all(|names| matches!(names, ScopedNames::One(_, _))));
}

#[test]
fn scoped_names_keep_duplicate_only_runs_inline_even_when_revisited() {
    let names = index_scoped_names([
        ((0, "run".to_string()), 1),
        ((0, "run".to_string()), 2),
        ((0, "run".to_string()), 3),
        ((1, "run".to_string()), 4),
        ((0, "run".to_string()), 5),
        ((0, "run".to_string()), 6),
    ]);
    assert_eq!(scoped_name(&names, 0, "run"), Some(&6));
    assert_eq!(scoped_name(&names, 1, "run"), Some(&4));
    assert!(names
        .values()
        .all(|names| matches!(names, ScopedNames::One(_, _))));
    assert!(index_scoped_names::<u32>([]).is_empty());
}

#[test]
fn scoped_names_reserve_dense_runs_and_extend_revisited_storage_shapes() {
    let entries = [((0, "first".to_string()), 0), ((1, "run".to_string()), 1)]
        .into_iter()
        .chain((0..128).map(|id| ((0, format!("name{id}")), id)))
        .chain([
            ((1, "stop".to_string()), 2),
            ((0, "first".to_string()), 3),
            ((0, "first".to_string()), 4),
            ((0, "last".to_string()), 5),
            ((1, "run".to_string()), 6),
        ]);
    let names = index_scoped_names(entries);
    for id in 0..128 {
        assert_eq!(scoped_name(&names, 0, &format!("name{id}")), Some(&id));
    }
    assert_eq!(scoped_name(&names, 0, "first"), Some(&4));
    assert_eq!(scoped_name(&names, 0, "last"), Some(&5));
    assert_eq!(scoped_name(&names, 1, "run"), Some(&6));
    assert_eq!(scoped_name(&names, 1, "stop"), Some(&2));
    assert_eq!(names.get(&0).unwrap().len(), 130);
    assert_eq!(names.get(&1).unwrap().len(), 2);
}

#[test]
fn scoped_names_promote_distinct_names_and_replace_duplicates_after_promotion() {
    let names = index_scoped_names([
        ((0, "run".to_string()), 1),
        ((0, "stop".to_string()), 2),
        ((0, "run".to_string()), 3),
        ((1, "run".to_string()), 4),
    ]);
    assert!(matches!(names.get(&0), Some(ScopedNames::Many(_))));
    assert!(matches!(names.get(&1), Some(ScopedNames::One(_, _))));
    assert_eq!(scoped_name(&names, 0, "run"), Some(&3));
    assert_eq!(scoped_name(&names, 0, "stop"), Some(&2));
    assert_eq!(scoped_name(&names, 1, "run"), Some(&4));
    assert!(scoped_name(&names, 0, "missing").is_none());
    assert_eq!(names.get(&0).unwrap().len(), 2);
    assert_eq!(names.get(&1).unwrap().len(), 1);
    let mut values = names
        .values()
        .flat_map(|names| names.values())
        .copied()
        .collect::<Vec<_>>();
    values.sort_unstable();
    assert_eq!(values, vec![2, 3, 4]);
}

#[test]
fn scoped_names_preserve_interleaved_promoted_scope_entries() {
    let names = index_scoped_names([
        ((0, "run".to_string()), 1),
        ((0, "stop".to_string()), 2),
        ((1, "run".to_string()), 3),
        ((0, "run".to_string()), 4),
        ((0, "reset".to_string()), 5),
        ((1, "run".to_string()), 6),
    ]);
    assert_eq!(scoped_name(&names, 0, "run"), Some(&4));
    assert_eq!(scoped_name(&names, 0, "stop"), Some(&2));
    assert_eq!(scoped_name(&names, 0, "reset"), Some(&5));
    assert_eq!(scoped_name(&names, 1, "run"), Some(&6));
    assert_eq!(names.values().map(|names| names.len()).sum::<usize>(), 4);
}

#[test]
fn cloned_scoped_names_keep_both_storage_shapes_independent() {
    let original = index_scoped_names([
        ((0, "run".to_string()), "root".to_string()),
        ((1, "run".to_string()), "nested".to_string()),
        ((1, "stop".to_string()), "stop".to_string()),
    ]);
    let mut cloned = original.clone();
    assert!(matches!(cloned.get(&0), Some(ScopedNames::One(_, _))));
    assert!(matches!(cloned.get(&1), Some(ScopedNames::Many(_))));
    for (scope, name) in [(0, "run"), (1, "run"), (1, "stop")] {
        assert_eq!(
            scoped_name(&cloned, scope, name),
            scoped_name(&original, scope, name)
        );
    }
    cloned
        .get_mut(&0)
        .unwrap()
        .insert("run".to_string(), "changed-root".to_string());
    cloned
        .get_mut(&1)
        .unwrap()
        .insert("run".to_string(), "changed-nested".to_string());
    assert_eq!(
        scoped_name(&original, 0, "run").map(String::as_str),
        Some("root")
    );
    assert_eq!(
        scoped_name(&original, 1, "run").map(String::as_str),
        Some("nested")
    );
    assert_eq!(
        scoped_name(&cloned, 0, "run").map(String::as_str),
        Some("changed-root")
    );
    assert_eq!(
        scoped_name(&cloned, 1, "run").map(String::as_str),
        Some("changed-nested")
    );
}

#[test]
fn scoped_callable_lookup_retains_shadowing_and_declaration_liveness() {
    let facts = crate::codebase::ts_source::facts::TsFileFacts {
        callable_bindings: vec![
            (0, "run".to_string(), CallableId(1)),
            (1, "run".to_string(), CallableId(2)),
            (2, "run".to_string(), CallableId(3)),
        ],
        callable_binding_declared_at: vec![
            (0, "run".to_string(), 0),
            (1, "run".to_string(), 20),
            (2, "run".to_string(), 40),
        ],
        lexical_scope_parents: vec![(0, None), (1, Some(0)), (2, Some(0)), (3, Some(1))],
        ..Default::default()
    };
    let index = CallableFileIndex::from_facts(&facts);
    assert_eq!(
        index.resolve_local_callable_id(Some(0), "run"),
        Some(CallableId(1))
    );
    assert_eq!(
        index.resolve_local_callable_id(Some(3), "run"),
        Some(CallableId(2))
    );
    assert_eq!(
        index.resolve_local_callable_id(Some(2), "run"),
        Some(CallableId(3))
    );
    assert!(!index.target_binding_live(1, "run", 19, Some(1), None));
    assert!(index.target_binding_live(1, "run", 20, Some(1), None));
    assert!(!index.target_binding_live(2, "run", 39, Some(2), None));
    assert!(index.target_binding_live(0, "run", 0, Some(0), None));
    assert_eq!(
        callable_id_for_scope(&facts, &index, "outer/run", Some(1)),
        Some(CallableId(2))
    );
    assert_eq!(
        callable_id_for_scope(&facts, &index, "outer/run", Some(2)),
        Some(CallableId(3))
    );
}

#[test]
fn lexical_name_tables_use_borrowed_probes() {
    let index = include_str!("../edge_calls.rs");
    for field in [
        "callable_bindings",
        "aliases",
        "binding_declared_at",
        "class_bindings",
    ] {
        assert!(index.contains(&format!("{field}: ScopedNameMap<")));
    }
    let lookup = include_str!("../edge_calls/scoped_names.rs");
    assert!(lookup.contains("scopes.get(&scope)?.get(name)"));
    assert!(!lookup.contains("to_string()"));
    assert!(!lookup.contains(".clone()"));
    // Many inserts must update in place; dense construction reserves from
    // an exact owned run instead of growing the inner map for each row.
    let insertion = lookup.split("fn insert(").nth(1).unwrap();
    let insertion = insertion.split("fn extend(").next().unwrap();
    assert!(!insertion.contains("std::mem::replace"));
    assert!(lookup.contains("names.reserve(rows.len())"));
    assert!(lookup.contains("ScopedNames::Many(rows.into_iter().collect())"));
    // Alias results and cycle detection still own strings; only table probes
    // must avoid allocating an owned (scope, name) lookup key.
    for probes in [
        include_str!("../edge_calls/alias_resolution.rs"),
        include_str!("../edge_calls/alias_liveness.rs"),
        include_str!("../edge_calls/class_resolution.rs"),
        include_str!("../edge_calls/liveness.rs"),
        include_str!("../edge_import_reachability_traversal.rs"),
    ] {
        assert!(probes.contains("scoped_name("));
        assert!(!probes.contains(".get(&("));
        assert!(!probes.contains(".contains_key(&("));
    }
}

#[test]
fn imported_class_alias_keeps_inherited_members_across_binding_scopes() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("scoped-class-alias"));
    let tsconfig = TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    let target = root.join("src/classes.mts");
    let facts = crate::codebase::ts_source::facts::collect_ts_facts(
        std::slice::from_ref(&target),
        crate::codebase::ts_source::facts::TsFactPlan {
            function_calls: true,
            imports: true,
            ..Default::default()
        },
    );
    let bindings = &facts.get(&target).unwrap().callable_bindings;
    let (scope, _, id) = bindings
        .iter()
        .find(|(_, name, _)| name == "Child")
        .unwrap();
    assert!(bindings.iter().any(|(alias_scope, name, alias_id)| {
        name == "Local" && alias_scope != scope && alias_id == id
    }));
    let graph = DepGraph::build_with_plan(
        &root,
        &tsconfig,
        GraphBuildPlan {
            calls: true,
            ..GraphBuildPlan::default()
        },
    )
    .unwrap();
    let calls = graph.deps_of(
        &[NodeId::file(root.join("src/consumer.mts"))],
        None,
        Some(&[EdgeKind::Call].into()),
    );
    assert!(calls.iter().any(|entry| {
        matches!(&entry.node, NodeId::Symbol { file, symbol, .. }
            if file.as_ref() == target.as_path() && symbol.as_ref() == "Base/run")
            && entry.via.contains(&EdgeKind::Call)
    }));
}
