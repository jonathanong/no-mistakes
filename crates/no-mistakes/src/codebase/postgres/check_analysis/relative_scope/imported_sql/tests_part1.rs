#[test]
fn module_const_sql_inlines_literals_and_leaves_computed_dynamic() {
    let calls = caller_calls(&run("module-const-sql"));
    let mut analyzed = Vec::new();
    let mut unknown = 0;
    for call in &calls {
        let sql = call.sql_text.clone().unwrap_or_default();
        if call.is_unanalyzable() {
            unknown += 1;
            assert!(!sql.contains('\0'), "{sql}");
            continue;
        }
        assert!(sql.contains("orders"), "{call:?}");
        analyzed.push(sql);
    }
    assert_eq!(unknown, 13, "{calls:#?}");
    let computed = calls.iter().find(|call| call.line == 100).unwrap();
    assert!(!computed.is_unanalyzable(), "{computed:#?}");
    assert_eq!(computed.variants.len(), 1);
    assert_eq!(
        computed.sql_text.as_deref(),
        Some(computed.variants[0].sql_text.as_str())
    );
    assert!(analyzed
        .iter()
        .any(|sql| sql == "SELECT id FROM orders WHERE account_id = $1"));
    assert!(analyzed
        .iter()
        .any(|sql| sql == "SELECT id, account_id FROM orders WHERE account_id = $1"));
    assert!(
        calls
            .iter()
            .any(|call| call.line == 69 && call.is_unanalyzable()),
        "an exported const builder is still a mutable object"
    );
    assert!(analyzed
        .iter()
        .any(|sql| sql == "SELECT id FROM orders WHERE true"));
}

#[test]
fn a_constant_hole_without_its_import_record_is_not_inlined() {
    // The walk already recorded the hole. Clearing the import makes the name
    // look local, which the filler must refuse instead of splicing a guess.
    let mut files = prepared("module-const-sql");
    let caller = files
        .keys()
        .find(|path| path.ends_with("src/caller.mts"))
        .expect("caller")
        .clone();
    Arc::make_mut(&mut files.get_mut(&caller).expect("caller").ts)
        .imported_bindings
        .clear();
    let calls = Mutex::new(BTreeMap::new());
    let resolve = resolver(&files, &calls);
    project(&mut files, resolve);
    let call = caller_calls(&files)
        .into_iter()
        .find(|call| call.line == 56)
        .expect("imported const");
    assert!(call.is_unanalyzable(), "{call:?}");
}

#[test]
fn a_shared_export_is_located_once_however_many_calls_import_it() {
    let mut files = prepared("imported-shared-export");
    assert!(has_candidates(&files));
    let calls = Mutex::new(BTreeMap::new());
    let resolve = resolver(&files, &calls);
    let observer = InvocationObserver::new(true);
    with_observer(Some(Arc::clone(&observer)), || project(&mut files, resolve));
    let sites: usize = files
        .values()
        .map(|file| file.postgres().embedded_sql[0].1.pending_imported.len())
        .sum();
    assert_eq!(sites, 3, "three call sites import the same export");
    assert_eq!(
        observer.snapshot().work["postgres.embedded.helper_resolutions"],
        1,
        "one (module, export) pair is located once, not once per call site"
    );
    let calls = calls.into_inner().unwrap();
    assert!(
        calls.values().all(|count| *count == 1),
        "each (importer, specifier) resolves once: {calls:?}"
    );
    let owner = variant_root("imported-shared-export").join("src/visibility-sql.mts");
    for (path, file) in &files {
        for call in &file.postgres().embedded_sql[0].1.calls {
            assert_eq!(call.kind, EmbeddedSqlKind::Composed, "{}", path.display());
            assert!(call
                .sql_text
                .as_deref()
                .unwrap()
                .contains("FROM view_active_plans"));
            assert!(call
                .sql_source_positions
                .iter()
                .any(|position| position.source_path.as_deref() == Some(owner.as_path())));
        }
    }
}

#[test]
fn the_request_pass_parses_each_file_once_and_locates_each_export_once() {
    use crate::codebase::check_facts::{collect_check_facts, CheckFactPlan};
    let saved = variant_root("imported-shared-export");
    let fixture = crate::test_support::materialize_saved_fixture(&saved);
    let root = normalize_path(fixture.path());
    let paths = ["first", "second", "index", "visibility-sql"]
        .map(|name| normalize_path(&root.join(format!("src/{name}.mts"))));
    let options = EmbeddedSqlOptions::configured("@example/db", &[]);
    let observer = InvocationObserver::new(true);
    crate::ast::begin_parse_count(&root);
    let facts = with_observer(Some(Arc::clone(&observer)), || {
        collect_check_facts(
            &root,
            paths.to_vec(),
            CheckFactPlan {
                embedded_sql: true,
                embedded_sql_options: vec![options.clone()],
                ..Default::default()
            },
        )
    });
    let counts = crate::ast::finish_parse_count(&root);
    for path in &paths {
        assert_eq!(counts.get(path), Some(&1), "{counts:#?}");
    }
    assert_eq!(
        observer.snapshot().work["postgres.embedded.helper_resolutions"],
        1,
        "three call sites, one (module, export) resolution"
    );
    for path in &paths[..2] {
        let calls = &facts.embedded_sql(path, &options).expect("facts").calls;
        assert!(!calls.is_empty());
        assert!(calls
            .iter()
            .all(|call| call.kind == EmbeddedSqlKind::Composed));
    }
}

#[test]
fn each_reexport_form_resolves_to_the_declaring_module() {
    let files = run("imported-reexport-forms");
    let expected = [
        ("d", "diamond: two star paths, one declaration"),
        ("e", "an explicit re-export outranks a star"),
        ("x", "a renamed default export"),
        ("r", "an import re-exported by name"),
    ];
    let calls = caller_calls(&files);
    assert_eq!(calls.len(), expected.len());
    for (call, (alias, why)) in calls.iter().zip(expected) {
        let sql = call.sql_text.as_deref().unwrap_or_default();
        assert!(
            sql.contains(&format!("FROM view_active_plans {alias} ")),
            "{why}: {sql}"
        );
        assert_eq!(call.kind, EmbeddedSqlKind::Composed, "{why}");
    }
}
