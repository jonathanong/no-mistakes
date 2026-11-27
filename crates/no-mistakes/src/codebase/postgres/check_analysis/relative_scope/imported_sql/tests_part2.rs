#[test]
fn the_reexport_depth_cap_is_inclusive() {
    assert_eq!(MAX_REEXPORT_HOPS, 8);
    for (name, resolved) in [
        ("imported-barrel-depth-cap", true),
        ("imported-barrel-past-depth-cap", false),
    ] {
        let calls = caller_calls(&run(name));
        assert_eq!(calls.len(), 1, "{name}");
        assert_eq!(
            calls[0].kind == EmbeddedSqlKind::Composed,
            resolved,
            "{name}: {:?}",
            calls[0]
        );
    }
}

#[test]
fn unproven_links_leave_every_call_dynamic() {
    for name in [
        "imported-barrel-cycle",
        "imported-barrel-ambiguous",
        "imported-barrel-past-depth-cap",
        "imported-unresolved",
        "imported-nonliteral-arg",
    ] {
        let calls = caller_calls(&run(name));
        assert!(!calls.is_empty(), "{name}");
        for call in calls {
            assert_eq!(call.kind, EmbeddedSqlKind::Dynamic, "{name}: {call:?}");
            assert!(
                call.sql_text
                    .as_deref()
                    .is_none_or(|sql| !sql.contains("view_active_plans")),
                "{name}: {call:?}"
            );
        }
    }
}

#[test]
fn a_namespace_reexport_marker_is_never_followed() {
    let mut files = prepared("imported-barrel");
    let index = files
        .keys()
        .find(|path| path.ends_with("src/helpers/index.mts"))
        .cloned()
        .expect("barrel");
    let facts = Arc::make_mut(&mut files.get_mut(&index).expect("facts").ts);
    // `export * as ownerVisibleSql from ...` shadows the name with a namespace object.
    facts
        .exported_bindings
        .push(crate::codebase::dependencies::extract::ExportedBinding {
            specifier: Some("../visibility/owner-sql.mts".to_string()),
            local: "*".to_string(),
            exported: "ownerVisibleSql".to_string(),
        });
    let calls = Mutex::new(BTreeMap::new());
    let resolve = resolver(&files, &calls);
    project(&mut files, resolve);
    let calls = caller_calls(&files);
    assert_eq!(calls[0].kind, EmbeddedSqlKind::Dynamic, "{:?}", calls[0]);
}

#[test]
fn a_re_exported_namespace_import_is_never_followed() {
    use crate::codebase::dependencies::extract::{
        ExportedBinding, ImportedBinding, ImportedBindingKind,
    };
    let mut files = prepared("imported-barrel");
    let index = files
        .keys()
        .find(|path| path.ends_with("src/helpers/index.mts"))
        .cloned()
        .expect("barrel");
    let facts = Arc::make_mut(&mut files.get_mut(&index).expect("facts").ts);
    // `import * as ownerVisibleSql ...; export { ownerVisibleSql }`
    facts.imported_bindings.push(ImportedBinding {
        specifier: "../visibility/owner-sql.mts".to_string(),
        local: "ownerVisibleSql".to_string(),
        imported: "*".to_string(),
        kind: ImportedBindingKind::Namespace,
        is_type_only: false,
    });
    facts.exported_bindings.push(ExportedBinding {
        specifier: None,
        local: "ownerVisibleSql".to_string(),
        exported: "ownerVisibleSql".to_string(),
    });
    let calls = Mutex::new(BTreeMap::new());
    let resolve = resolver(&files, &calls);
    project(&mut files, resolve);
    assert_eq!(caller_calls(&files)[0].kind, EmbeddedSqlKind::Dynamic);
}

#[test]
fn a_call_whose_import_binding_is_missing_stays_dynamic() {
    let mut files = prepared("imported-literal-identifiers");
    let caller = files
        .keys()
        .find(|path| path.ends_with("src/caller.mts"))
        .cloned()
        .expect("caller");
    let facts = Arc::make_mut(&mut files.get_mut(&caller).expect("facts").ts);
    facts.imported_bindings.clear();
    let calls = Mutex::new(BTreeMap::new());
    let resolve = resolver(&files, &calls);
    project(&mut files, resolve);
    assert_eq!(caller_calls(&files)[0].kind, EmbeddedSqlKind::Dynamic);
    assert!(calls.into_inner().unwrap().is_empty(), "nothing to resolve");
}

#[test]
fn a_re_export_from_an_unresolved_module_is_unprovable() {
    let mut files = prepared("imported-barrel-ambiguous");
    let index = files
        .keys()
        .find(|path| path.ends_with("src/index.mts"))
        .cloned()
        .expect("barrel");
    let facts = Arc::make_mut(&mut files.get_mut(&index).expect("facts").ts);
    facts
        .star_reexport_specifiers
        .push("./gone.mts".to_string());
    let calls = Mutex::new(BTreeMap::new());
    let mut locator = Locator::new(&files, resolver(&files, &calls));
    // The missing module might declare the name, so it is not `Absent`.
    assert_eq!(locator.locate(&index, "neverDeclared"), Located::Unknown);
}

#[test]
fn the_locator_distinguishes_absent_from_unprovable() {
    let files = prepared("imported-barrel-ambiguous");
    let calls = Mutex::new(BTreeMap::new());
    let mut locator = Locator::new(&files, resolver(&files, &calls));
    let src = variant_root("imported-barrel-ambiguous").join("src");
    let index = src.join("index.mts");
    // Two different declarations behind `export *` make the name ambiguous.
    assert_eq!(locator.locate(&index, "visibleSql"), Located::Unknown);
    assert_eq!(locator.locate(&index, "neverDeclared"), Located::Absent);
    // `export *` never forwards a default export.
    assert_eq!(locator.locate(&index, "default"), Located::Absent);
    assert_eq!(
        locator.locate(&src.join("one.mts"), "visibleSql"),
        Located::Found(src.join("one.mts"), "visibleSql".to_string())
    );
    // A module outside the scanned files cannot be proven either way.
    assert_eq!(
        locator.locate(Path::new("/nowhere/outside.mts"), "visibleSql"),
        Located::Unknown
    );
    assert_eq!(
        locator.target("./one.mts", &index),
        Some(src.join("one.mts"))
    );
    assert_eq!(locator.target("./missing.mts", &index), None);
}

#[test]
fn a_file_without_pending_helpers_is_not_a_candidate() {
    let files = prepared("same-file-literal-identifiers");
    assert!(!has_candidates(&files));
}
